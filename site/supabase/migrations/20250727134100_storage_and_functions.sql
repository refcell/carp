-- Storage setup and database functions for package management

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

-- Create function to get agent details with version information
CREATE OR REPLACE FUNCTION public.get_agent_details(agent_name TEXT, agent_author TEXT DEFAULT '')
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
  license TEXT,
  homepage TEXT,
  repository TEXT,
  readme TEXT,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  versions JSONB
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
BEGIN
  RETURN QUERY
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
    a.license,
    a.homepage,
    a.repository,
    a.readme,
    a.created_at,
    a.updated_at,
    COALESCE(
      (
        SELECT jsonb_agg(
          jsonb_build_object(
            'version', av.version,
            'description', av.description,
            'changelog', av.changelog,
            'download_count', av.download_count,
            'is_pre_release', av.is_pre_release,
            'yanked', av.yanked,
            'yanked_reason', av.yanked_reason,
            'created_at', av.created_at,
            'package_size', av.package_size,
            'checksum', av.checksum
          ) ORDER BY av.created_at DESC
        )
        FROM public.agent_versions av
        WHERE av.agent_id = a.id
      ),
      '[]'::jsonb
    ) as versions
  FROM public.agents a
  WHERE a.name = agent_name
    AND a.is_public = true
    AND (agent_author = '' OR LOWER(a.author_name) = LOWER(agent_author));
END;
$$;

-- Create function to increment download counts
CREATE OR REPLACE FUNCTION public.record_download(
  agent_name TEXT,
  version_text TEXT DEFAULT '',
  user_agent_text TEXT DEFAULT '',
  ip_addr INET DEFAULT NULL
)
RETURNS BOOLEAN
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
DECLARE
  agent_record RECORD;
  version_record RECORD;
  package_record RECORD;
BEGIN
  -- Find the agent
  SELECT id, download_count INTO agent_record
  FROM public.agents 
  WHERE name = agent_name AND is_public = true;
  
  IF NOT FOUND THEN
    RETURN FALSE;
  END IF;
  
  -- Find the version (use latest if not specified)
  IF version_text = '' THEN
    SELECT av.id, av.download_count, av.package_size INTO version_record
    FROM public.agent_versions av
    WHERE av.agent_id = agent_record.id
    ORDER BY av.created_at DESC
    LIMIT 1;
  ELSE
    SELECT av.id, av.download_count, av.package_size INTO version_record
    FROM public.agent_versions av
    WHERE av.agent_id = agent_record.id AND av.version = version_text;
  END IF;
  
  IF NOT FOUND THEN
    RETURN FALSE;
  END IF;
  
  -- Find the package
  SELECT id, file_size INTO package_record
  FROM public.agent_packages ap
  WHERE ap.version_id = version_record.id
  LIMIT 1;
  
  -- Record the download
  INSERT INTO public.download_stats (
    agent_id, 
    version_id, 
    package_id, 
    user_id, 
    ip_address, 
    user_agent,
    file_size
  ) VALUES (
    agent_record.id,
    version_record.id,
    package_record.id,
    auth.uid(),
    COALESCE(ip_addr, inet_client_addr()),
    user_agent_text,
    COALESCE(package_record.file_size, version_record.package_size)
  );
  
  -- Update counters
  UPDATE public.agents 
  SET download_count = download_count + 1
  WHERE id = agent_record.id;
  
  UPDATE public.agent_versions
  SET download_count = download_count + 1
  WHERE id = version_record.id;
  
  RETURN TRUE;
END;
$$;

-- Create function for secure agent creation
CREATE OR REPLACE FUNCTION public.create_agent(
  agent_name TEXT,
  description TEXT,
  author_name TEXT DEFAULT '',
  tags TEXT[] DEFAULT ARRAY[]::TEXT[],
  keywords TEXT[] DEFAULT ARRAY[]::TEXT[],
  license TEXT DEFAULT '',
  homepage TEXT DEFAULT '',
  repository TEXT DEFAULT '',
  readme TEXT DEFAULT '',
  is_public BOOLEAN DEFAULT true
)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
DECLARE
  user_id UUID;
  agent_id UUID;
  profile_record public.profiles;
BEGIN
  -- Get current user ID (from auth or API token)
  user_id := auth.uid();
  
  IF user_id IS NULL THEN
    -- Try to get user from API token
    SELECT vat.user_id INTO user_id
    FROM public.validate_api_token(
      COALESCE(
        current_setting('request.headers', true)::json->>'authorization',
        ''
      )
    ) vat
    WHERE 'write' = ANY(vat.scopes);
  END IF;
  
  IF user_id IS NULL THEN
    RETURN jsonb_build_object(
      'success', false,
      'error', 'Authentication required'
    );
  END IF;
  
  -- Get user profile for default author name
  SELECT * INTO profile_record
  FROM public.profiles 
  WHERE profiles.user_id = create_agent.user_id;
  
  -- Check if agent name is already taken
  IF EXISTS(SELECT 1 FROM public.agents WHERE name = agent_name) THEN
    RETURN jsonb_build_object(
      'success', false,
      'error', 'Agent name already exists'
    );
  END IF;
  
  -- Create the agent
  INSERT INTO public.agents (
    user_id,
    name,
    description,
    author_name,
    tags,
    keywords,
    license,
    homepage,
    repository,
    readme,
    is_public,
    definition
  ) VALUES (
    user_id,
    agent_name,
    description,
    COALESCE(NULLIF(author_name, ''), profile_record.display_name, profile_record.github_username, 'Unknown'),
    tags,
    keywords,
    license,
    homepage,
    repository,
    readme,
    is_public,
    '{}'::jsonb
  ) RETURNING id INTO agent_id;
  
  RETURN jsonb_build_object(
    'success', true,
    'agent_id', agent_id,
    'message', 'Agent created successfully'
  );
END;
$$;

-- Create secure function for publishing new agent versions
CREATE OR REPLACE FUNCTION public.publish_agent_version(
  agent_name TEXT,
  version TEXT,
  description TEXT DEFAULT '',
  changelog TEXT DEFAULT '',
  definition_data JSONB DEFAULT '{}'::jsonb,
  package_data JSONB DEFAULT '{}'::jsonb -- {file_name, file_size, checksum, content_type}
)
RETURNS JSONB
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
DECLARE
  agent_record public.agents;
  version_id UUID;
  package_id UUID;
  result JSONB;
BEGIN
  -- Find the agent (must be owned by current user or via API token)
  SELECT * INTO agent_record
  FROM public.agents 
  WHERE name = agent_name 
    AND (
      user_id = auth.uid() OR
      EXISTS(
        SELECT 1 FROM public.validate_api_token(
          COALESCE(
            current_setting('request.headers', true)::json->>'authorization',
            ''
          )
        ) vat WHERE vat.user_id = agents.user_id AND 'write' = ANY(vat.scopes)
      )
    );
  
  IF NOT FOUND THEN
    RETURN jsonb_build_object(
      'success', false,
      'error', 'Agent not found or access denied'
    );
  END IF;
  
  -- Check if version already exists
  IF EXISTS(
    SELECT 1 FROM public.agent_versions 
    WHERE agent_id = agent_record.id AND agent_versions.version = publish_agent_version.version
  ) THEN
    RETURN jsonb_build_object(
      'success', false,
      'error', 'Version already exists'
    );
  END IF;
  
  -- Create the version
  INSERT INTO public.agent_versions (
    agent_id, 
    version, 
    description, 
    changelog, 
    definition,
    package_size,
    checksum
  ) VALUES (
    agent_record.id,
    version,
    description,
    changelog,
    definition_data,
    (package_data->>'file_size')::bigint,
    package_data->>'checksum'
  ) RETURNING id INTO version_id;
  
  -- Create package record if package data provided
  IF package_data != '{}' THEN
    INSERT INTO public.agent_packages (
      version_id,
      file_name,
      file_path,
      content_type,
      file_size,
      checksum
    ) VALUES (
      version_id,
      package_data->>'file_name',
      format('%s/%s/%s/%s', 
        agent_record.user_id,
        agent_record.name,
        version,
        package_data->>'file_name'
      ),
      COALESCE(package_data->>'content_type', 'application/gzip'),
      (package_data->>'file_size')::bigint,
      package_data->>'checksum'
    ) RETURNING id INTO package_id;
  END IF;
  
  -- Update agent's current version if this is the latest
  UPDATE public.agents 
  SET 
    current_version = version,
    latest_version_id = version_id,
    updated_at = now()
  WHERE id = agent_record.id;
  
  RETURN jsonb_build_object(
    'success', true,
    'version_id', version_id,
    'package_id', package_id,
    'message', 'Version published successfully'
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
GRANT EXECUTE ON FUNCTION public.get_agent_details TO anon, authenticated;
GRANT EXECUTE ON FUNCTION public.record_download TO anon, authenticated;
GRANT EXECUTE ON FUNCTION public.validate_api_token TO anon, authenticated;
GRANT EXECUTE ON FUNCTION public.publish_agent_version TO authenticated;
GRANT EXECUTE ON FUNCTION public.create_agent TO authenticated;