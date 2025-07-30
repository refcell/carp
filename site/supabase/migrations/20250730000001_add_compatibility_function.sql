-- Add a compatibility function to handle the cached PostgREST schema
-- This function accepts the parameters that PostgREST is expecting and delegates to our correct function

CREATE OR REPLACE FUNCTION public.get_agent_download_info(
  p_agent_name TEXT,
  p_user_id UUID,
  p_version_text TEXT
)
RETURNS TABLE (
  agent_name TEXT,
  version TEXT,
  file_path TEXT,
  checksum TEXT,
  file_size BIGINT
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
BEGIN
  -- Simply delegate to the correct function, ignoring the p_user_id parameter
  -- since downloads are public for published agents
  RETURN QUERY 
  SELECT * FROM public.get_agent_download_info(p_agent_name, p_version_text);
END;
$$;

-- Grant execute permissions
GRANT EXECUTE ON FUNCTION public.get_agent_download_info(TEXT, UUID, TEXT) TO anon, authenticated;