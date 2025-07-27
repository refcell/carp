-- Recreate functions and storage for package management

-- Create storage bucket for agent packages (only if it doesn't exist)
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM storage.buckets WHERE id = 'agent-packages') THEN
    INSERT INTO storage.buckets (id, name, public, file_size_limit, allowed_mime_types)
    VALUES (
      'agent-packages',
      'agent-packages',
      false, -- private bucket, access controlled by RLS
      104857600, -- 100MB limit per file
      ARRAY['application/gzip', 'application/x-gzip', 'application/tar+gzip', 'application/zip']
    );
  END IF;
END
$$;

-- Create storage policies for agent packages bucket
CREATE POLICY "Users can upload packages for their own agents"
ON storage.objects 
FOR INSERT 
WITH CHECK (
  bucket_id = 'agent-packages' AND 
  auth.uid() IS NOT NULL AND
  -- Ensure the path follows the pattern: user_id/agent_name/version/filename
  (storage.foldername(name))[1] = auth.uid()::text
);

CREATE POLICY "Users can view packages for public agents or their own agents"
ON storage.objects 
FOR SELECT 
USING (
  bucket_id = 'agent-packages' AND (
    -- Allow access to own packages
    (storage.foldername(name))[1] = auth.uid()::text OR
    -- Allow access to public agent packages
    EXISTS (
      SELECT 1 FROM public.agents a
      WHERE a.user_id::text = (storage.foldername(name))[1]
      AND a.name = (storage.foldername(name))[2]
      AND a.is_public = true
    )
  )
);

CREATE POLICY "Users can update their own agent packages"
ON storage.objects 
FOR UPDATE 
USING (
  bucket_id = 'agent-packages' AND 
  (storage.foldername(name))[1] = auth.uid()::text
);

CREATE POLICY "Users can delete their own agent packages"
ON storage.objects 
FOR DELETE 
USING (
  bucket_id = 'agent-packages' AND 
  (storage.foldername(name))[1] = auth.uid()::text
);

-- Create function to validate API token
CREATE OR REPLACE FUNCTION public.validate_api_token(token_hash TEXT)
RETURNS TABLE (
  user_id UUID,
  scopes TEXT[]
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
BEGIN
  -- Update last_used_at and return user info
  UPDATE public.api_tokens 
  SET 
    last_used_at = now(),
    last_used_ip = inet_client_addr()
  WHERE 
    api_tokens.token_hash = validate_api_token.token_hash
    AND is_active = true 
    AND (expires_at IS NULL OR expires_at > now());
    
  RETURN QUERY
  SELECT 
    api_tokens.user_id,
    api_tokens.scopes
  FROM public.api_tokens
  WHERE 
    api_tokens.token_hash = validate_api_token.token_hash
    AND is_active = true 
    AND (expires_at IS NULL OR expires_at > now());
END;
$$;

-- Create an immutable function for search text generation
CREATE OR REPLACE FUNCTION public.agent_search_text(
  name TEXT,
  description TEXT,
  author_name TEXT,
  tags TEXT[],
  keywords TEXT[]
) RETURNS TEXT
LANGUAGE sql
IMMUTABLE
SET search_path = ''
AS $$
  SELECT COALESCE(name, '') || ' ' || 
         COALESCE(description, '') || ' ' || 
         COALESCE(author_name, '') || ' ' ||
         COALESCE(array_to_string(tags, ' '), '') || ' ' ||
         COALESCE(array_to_string(keywords, ' '), '')
$$;

-- Update the existing search index to use the immutable function
-- First drop the old index name and create the new one
DROP INDEX IF EXISTS idx_agents_name_search;
CREATE INDEX IF NOT EXISTS idx_agents_search ON public.agents USING GIN(
  to_tsvector('english', public.agent_search_text(name, description, author_name, tags, keywords))
);

-- Create function for advanced agent search
CREATE OR REPLACE FUNCTION public.search_agents(
  search_query TEXT DEFAULT '',
  tags_filter TEXT[] DEFAULT ARRAY[]::TEXT[],
  author_filter TEXT DEFAULT '',
  sort_by TEXT DEFAULT 'relevance', -- relevance, downloads, created_at, updated_at, rating
  sort_order TEXT DEFAULT 'desc',
  page_num INTEGER DEFAULT 1,
  page_size INTEGER DEFAULT 20
)
RETURNS TABLE (
  id UUID,
  name TEXT,
  description TEXT,
  author_name TEXT,
  current_version TEXT,
  tags TEXT[],
  keywords TEXT[],
  download_count BIGINT,
  view_count INTEGER,
  average_rating NUMERIC,
  rating_count BIGINT,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  total_count BIGINT
) 
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
DECLARE
  base_query TEXT;
  where_conditions TEXT[] := ARRAY[]::TEXT[];
  order_clause TEXT;
  offset_val INTEGER;
BEGIN
  -- Input validation
  IF page_num < 1 THEN page_num := 1; END IF;
  IF page_size < 1 OR page_size > 100 THEN page_size := 20; END IF;
  offset_val := (page_num - 1) * page_size;
  
  -- Build WHERE conditions
  where_conditions := ARRAY['a.is_public = true'];
  
  -- Text search condition
  IF search_query != '' THEN
    where_conditions := array_append(
      where_conditions,
      format('to_tsvector(''english'', public.agent_search_text(a.name, a.description, a.author_name, a.tags, a.keywords)) @@ plainto_tsquery(''english'', %L)', search_query)
    );
  END IF;
  
  -- Tags filter
  IF array_length(tags_filter, 1) > 0 THEN
    where_conditions := array_append(
      where_conditions,
      format('a.tags && %L', tags_filter)
    );
  END IF;
  
  -- Author filter
  IF author_filter != '' THEN
    where_conditions := array_append(
      where_conditions,
      format('LOWER(a.author_name) = LOWER(%L)', author_filter)
    );
  END IF;
  
  -- Build ORDER BY clause
  CASE sort_by
    WHEN 'downloads' THEN order_clause := 'a.download_count';
    WHEN 'created_at' THEN order_clause := 'a.created_at';
    WHEN 'updated_at' THEN order_clause := 'a.updated_at';
    WHEN 'rating' THEN order_clause := 'COALESCE(AVG(ar.rating), 0)';
    WHEN 'name' THEN order_clause := 'LOWER(a.name)';
    ELSE -- relevance
      IF search_query != '' THEN
        order_clause := 'ts_rank(to_tsvector(''english'', public.agent_search_text(a.name, a.description, a.author_name, a.tags, a.keywords)), plainto_tsquery(''english'', ''' || search_query || '''))';
      ELSE
        order_clause := 'a.created_at';
      END IF;
  END CASE;
  
  IF UPPER(sort_order) = 'ASC' THEN
    order_clause := order_clause || ' ASC';
  ELSE
    order_clause := order_clause || ' DESC';
  END IF;
  
  -- Build and execute the query
  RETURN QUERY EXECUTE format('
    WITH search_results AS (
      SELECT 
        a.id,
        a.name,
        a.description,
        a.author_name,
        a.current_version,
        a.tags,
        a.keywords,
        a.download_count,
        a.view_count,
        COALESCE(AVG(ar.rating), 0) as average_rating,
        COUNT(ar.id) as rating_count,
        a.created_at,
        a.updated_at,
        COUNT(*) OVER() as total_count
      FROM public.agents a
      LEFT JOIN public.agent_ratings ar ON a.id = ar.agent_id
      WHERE %s
      GROUP BY a.id, a.name, a.description, a.author_name, a.current_version, 
               a.tags, a.keywords, a.download_count, a.view_count, a.created_at, a.updated_at
      ORDER BY %s
      LIMIT %s OFFSET %s
    )
    SELECT * FROM search_results',
    array_to_string(where_conditions, ' AND '),
    order_clause,
    page_size,
    offset_val
  );
END;
$$;

-- Grant necessary permissions for API functions
GRANT USAGE ON SCHEMA public TO anon, authenticated;
GRANT SELECT ON public.agents TO anon, authenticated;
GRANT SELECT ON public.agent_versions TO anon, authenticated;
GRANT SELECT ON public.agent_packages TO anon, authenticated;
GRANT SELECT ON public.profiles TO anon, authenticated;
GRANT EXECUTE ON FUNCTION public.search_agents TO anon, authenticated;
GRANT EXECUTE ON FUNCTION public.validate_api_token TO anon, authenticated;