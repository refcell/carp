-- Utility functions and performance optimizations

-- Create function to get popular tags
CREATE OR REPLACE FUNCTION public.get_popular_tags(limit_count INTEGER DEFAULT 20)
RETURNS TABLE (tag TEXT, count BIGINT)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
BEGIN
  RETURN QUERY
  SELECT 
    unnest(tags) as tag,
    COUNT(*) as count
  FROM public.agents
  WHERE is_public = true AND tags IS NOT NULL
  GROUP BY tag
  ORDER BY count DESC, tag ASC
  LIMIT limit_count;
END;
$$;

-- Create function to get user's agent statistics
CREATE OR REPLACE FUNCTION public.get_user_agent_stats(target_user_id UUID DEFAULT NULL)
RETURNS TABLE (
  total_agents BIGINT,
  total_downloads BIGINT,
  total_versions BIGINT,
  public_agents BIGINT,
  private_agents BIGINT,
  average_rating NUMERIC,
  total_ratings BIGINT
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
DECLARE
  user_id UUID;
BEGIN
  user_id := COALESCE(target_user_id, auth.uid());
  
  IF user_id IS NULL THEN
    RETURN;
  END IF;
  
  RETURN QUERY
  SELECT 
    COUNT(a.id) as total_agents,
    COALESCE(SUM(a.download_count), 0) as total_downloads,
    COUNT(av.id) as total_versions,
    COUNT(a.id) FILTER (WHERE a.is_public = true) as public_agents,
    COUNT(a.id) FILTER (WHERE a.is_public = false) as private_agents,
    COALESCE(AVG(ar.rating), 0) as average_rating,
    COUNT(ar.id) as total_ratings
  FROM public.agents a
  LEFT JOIN public.agent_versions av ON a.id = av.agent_id
  LEFT JOIN public.agent_ratings ar ON a.id = ar.agent_id
  WHERE a.user_id = get_user_agent_stats.user_id;
END;
$$;

-- Create function to get agent dependencies (if stored in definition)
CREATE OR REPLACE FUNCTION public.get_agent_dependencies(agent_name TEXT)
RETURNS TABLE (
  dependency_name TEXT,
  version_constraint TEXT,
  dependency_type TEXT
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
BEGIN
  RETURN QUERY
  SELECT 
    deps.value->>'name' as dependency_name,
    deps.value->>'version' as version_constraint,
    deps.value->>'type' as dependency_type
  FROM public.agents a,
  jsonb_array_elements(a.definition->'dependencies') as deps
  WHERE a.name = agent_name 
    AND a.is_public = true
    AND a.definition ? 'dependencies';
END;
$$;

-- Create rate limiting function
CREATE OR REPLACE FUNCTION public.check_rate_limit(
  identifier TEXT,
  endpoint TEXT,
  max_requests INTEGER DEFAULT 100,
  window_minutes INTEGER DEFAULT 60
)
RETURNS BOOLEAN
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
DECLARE
  window_start TIMESTAMP WITH TIME ZONE;
  current_count INTEGER;
BEGIN
  -- Calculate window start time
  window_start := date_trunc('hour', now()) + 
    (EXTRACT(MINUTE FROM now())::integer / window_minutes) * 
    (window_minutes * interval '1 minute');
  
  -- Clean up old entries
  DELETE FROM public.rate_limits 
  WHERE window_start < now() - (window_minutes * 2 * interval '1 minute');
  
  -- Get or create current window count
  INSERT INTO public.rate_limits (identifier, endpoint, window_start, request_count)
  VALUES (identifier, endpoint, window_start, 1)
  ON CONFLICT (identifier, endpoint, window_start)
  DO UPDATE SET 
    request_count = rate_limits.request_count + 1,
    created_at = now()
  RETURNING request_count INTO current_count;
  
  RETURN current_count <= max_requests;
END;
$$;

-- Create function to log webhook events
CREATE OR REPLACE FUNCTION public.log_webhook_event(
  event_type TEXT,
  agent_id UUID DEFAULT NULL,
  version_id UUID DEFAULT NULL,
  user_id UUID DEFAULT NULL,
  payload JSONB DEFAULT '{}'::jsonb
)
RETURNS UUID
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
DECLARE
  event_id UUID;
BEGIN
  INSERT INTO public.webhook_events (
    event_type,
    agent_id,
    version_id,
    user_id,
    payload
  ) VALUES (
    event_type,
    agent_id,
    version_id,
    user_id,
    payload
  ) RETURNING id INTO event_id;
  
  RETURN event_id;
END;
$$;

-- Create trigger to log agent publication events
CREATE OR REPLACE FUNCTION public.trigger_agent_published()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
BEGIN
  PERFORM public.log_webhook_event(
    'agent.published',
    NEW.agent_id,
    NEW.id,
    auth.uid(),
    jsonb_build_object(
      'agent_name', (SELECT name FROM public.agents WHERE id = NEW.agent_id),
      'version', NEW.version,
      'author', (SELECT author_name FROM public.agents WHERE id = NEW.agent_id)
    )
  );
  RETURN NEW;
END;
$$;

-- Create the trigger
DROP TRIGGER IF EXISTS agent_version_published ON public.agent_versions;
CREATE TRIGGER agent_version_published
  AFTER INSERT ON public.agent_versions
  FOR EACH ROW
  EXECUTE FUNCTION public.trigger_agent_published();

-- Create materialized view for trending agents (updated periodically)
CREATE MATERIALIZED VIEW public.trending_agents AS
SELECT 
  a.id,
  a.name,
  a.description,
  a.author_name,
  a.current_version,
  a.tags,
  a.download_count,
  a.view_count,
  a.created_at,
  -- Calculate trending score based on recent downloads and ratings
  (
    COALESCE(recent_downloads.count, 0) * 0.6 + 
    COALESCE(avg_rating.rating, 0) * 0.3 + 
    COALESCE(rating_count.count, 0) * 0.1
  ) as trending_score,
  recent_downloads.count as recent_downloads,
  avg_rating.rating as average_rating,
  rating_count.count as rating_count
FROM public.agents a
LEFT JOIN (
  SELECT 
    ds.agent_id,
    COUNT(*) as count
  FROM public.download_stats ds
  WHERE ds.downloaded_at > now() - interval '7 days'
  GROUP BY ds.agent_id
) recent_downloads ON a.id = recent_downloads.agent_id
LEFT JOIN (
  SELECT 
    ar.agent_id,
    AVG(ar.rating) as rating
  FROM public.agent_ratings ar
  GROUP BY ar.agent_id
) avg_rating ON a.id = avg_rating.agent_id
LEFT JOIN (
  SELECT 
    ar.agent_id,
    COUNT(*) as count
  FROM public.agent_ratings ar
  GROUP BY ar.agent_id
) rating_count ON a.id = rating_count.agent_id
WHERE a.is_public = true
ORDER BY trending_score DESC;

-- Create indexes on the materialized view
CREATE INDEX IF NOT EXISTS idx_trending_agents_score ON public.trending_agents(trending_score DESC);
CREATE INDEX IF NOT EXISTS idx_trending_agents_name ON public.trending_agents(name);
CREATE INDEX IF NOT EXISTS idx_trending_agents_author ON public.trending_agents(author_name);

-- Create function to refresh trending agents (to be called periodically)
CREATE OR REPLACE FUNCTION public.refresh_trending_agents()
RETURNS VOID
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
BEGIN
  REFRESH MATERIALIZED VIEW public.trending_agents;
END;
$$;

-- Create cleanup function for old data
CREATE OR REPLACE FUNCTION public.cleanup_old_data()
RETURNS VOID
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
BEGIN
  -- Clean up old download stats (keep 1 year)
  DELETE FROM public.download_stats 
  WHERE downloaded_at < now() - interval '1 year';
  
  -- Clean up old rate limit entries (keep 1 day)
  DELETE FROM public.rate_limits 
  WHERE created_at < now() - interval '1 day';
  
  -- Clean up processed webhook events (keep 30 days)
  DELETE FROM public.webhook_events 
  WHERE processed = true AND created_at < now() - interval '30 days';
  
  -- Clean up expired API tokens
  DELETE FROM public.api_tokens 
  WHERE expires_at IS NOT NULL AND expires_at < now();
END;
$$;

-- Create view for agent statistics
CREATE VIEW public.agent_stats AS
SELECT 
  a.id as agent_id,
  a.name,
  a.author_name,
  a.download_count as total_downloads,
  COUNT(DISTINCT av.id) as version_count,
  COUNT(DISTINCT ar.id) as rating_count,
  COALESCE(AVG(ar.rating), 0) as average_rating,
  COUNT(DISTINCT uf.follower_id) as follower_count,
  MAX(av.created_at) as last_version_at,
  COUNT(DISTINCT ds.id) FILTER (WHERE ds.downloaded_at >= NOW() - INTERVAL '30 days') as downloads_last_30_days
FROM public.agents a
LEFT JOIN public.agent_versions av ON a.id = av.agent_id
LEFT JOIN public.agent_ratings ar ON a.id = ar.agent_id
LEFT JOIN public.user_follows uf ON a.id = uf.following_agent_id
LEFT JOIN public.download_stats ds ON a.id = ds.agent_id
GROUP BY a.id, a.name, a.author_name, a.download_count;

-- Create additional composite indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_agents_public_downloads 
ON public.agents(is_public, download_count DESC) 
WHERE is_public = true;

CREATE INDEX IF NOT EXISTS idx_agents_public_created 
ON public.agents(is_public, created_at DESC) 
WHERE is_public = true;

CREATE INDEX IF NOT EXISTS idx_agents_public_updated 
ON public.agents(is_public, updated_at DESC) 
WHERE is_public = true;

CREATE INDEX IF NOT EXISTS idx_agents_user_name 
ON public.agents(user_id, name);

CREATE INDEX IF NOT EXISTS idx_agent_versions_agent_created 
ON public.agent_versions(agent_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_agent_versions_not_yanked 
ON public.agent_versions(agent_id, created_at DESC) 
WHERE yanked = false;

CREATE INDEX IF NOT EXISTS idx_download_stats_agent_date 
ON public.download_stats(agent_id, downloaded_at DESC);

-- Create partial indexes for active tokens
CREATE INDEX IF NOT EXISTS idx_api_tokens_active 
ON public.api_tokens(user_id, token_hash) 
WHERE is_active = true;

-- Grant execute permissions for new functions
GRANT EXECUTE ON FUNCTION public.get_popular_tags TO anon, authenticated;
GRANT EXECUTE ON FUNCTION public.get_user_agent_stats TO authenticated;
GRANT EXECUTE ON FUNCTION public.get_agent_dependencies TO anon, authenticated;
GRANT EXECUTE ON FUNCTION public.check_rate_limit TO anon, authenticated;
GRANT EXECUTE ON FUNCTION public.refresh_trending_agents TO authenticated;
GRANT EXECUTE ON FUNCTION public.cleanup_old_data TO authenticated;
GRANT SELECT ON public.trending_agents TO anon, authenticated;
GRANT SELECT ON public.agent_stats TO anon, authenticated;