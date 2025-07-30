-- Enhanced get_agent_download_info function with additional fields
-- Drop all existing function overloads
DROP FUNCTION IF EXISTS public.get_agent_download_info(TEXT, TEXT);
DROP FUNCTION IF EXISTS public.get_agent_download_info(TEXT, UUID, TEXT);
DROP FUNCTION IF EXISTS public.get_agent_download_info(TEXT, TEXT, UUID);

-- Create enhanced function that returns all required fields
CREATE OR REPLACE FUNCTION public.get_agent_download_info(
  p_agent_name TEXT,
  p_version_text TEXT DEFAULT ''
)
RETURNS TABLE (
  agent_id TEXT,
  agent_name TEXT,
  author TEXT,
  version TEXT,
  file_path TEXT,
  checksum TEXT,
  file_size BIGINT,
  definition JSONB
)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
DECLARE
  agent_record RECORD;
  version_record RECORD;
  package_record RECORD;
  author_info RECORD;
BEGIN
  -- Find the agent
  SELECT a.id, a.name, a.user_id INTO agent_record
  FROM public.agents a
  WHERE a.name = p_agent_name AND a.is_public = true;
  
  IF NOT FOUND THEN
    RETURN;
  END IF;
  
  -- Get author information
  SELECT 
    COALESCE(p.display_name, p.github_username, 'Unknown') as display_name
  INTO author_info
  FROM public.profiles p
  WHERE p.user_id = agent_record.user_id
  LIMIT 1;
  
  -- Find the version (use latest if not specified or "latest")
  IF p_version_text = '' OR p_version_text = 'latest' THEN
    -- Get the latest version
    SELECT av.id, av.version, av.checksum, av.package_size, av.definition INTO version_record
    FROM public.agent_versions av
    WHERE av.agent_id = agent_record.id
      AND av.yanked = false
    ORDER BY av.created_at DESC
    LIMIT 1;
  ELSE
    -- Get specific version
    SELECT av.id, av.version, av.checksum, av.package_size, av.definition INTO version_record
    FROM public.agent_versions av
    WHERE av.agent_id = agent_record.id 
      AND av.version = p_version_text
      AND av.yanked = false;
  END IF;
  
  IF NOT FOUND THEN
    RETURN;
  END IF;
  
  -- Find the package file
  SELECT file_path, checksum, file_size INTO package_record
  FROM public.agent_packages ap
  WHERE ap.version_id = version_record.id
    AND ap.upload_completed = true
  ORDER BY ap.created_at DESC
  LIMIT 1;
  
  IF NOT FOUND THEN
    RETURN;
  END IF;
  
  -- Return the download information
  RETURN QUERY SELECT 
    agent_record.id::TEXT,
    agent_record.name::TEXT,
    COALESCE(author_info.display_name, 'Unknown')::TEXT,
    version_record.version::TEXT,
    package_record.file_path::TEXT,
    COALESCE(package_record.checksum, version_record.checksum, '')::TEXT,
    COALESCE(package_record.file_size, version_record.package_size, 0)::BIGINT,
    version_record.definition::JSONB;
END;
$$;

-- Grant execute permissions
GRANT EXECUTE ON FUNCTION public.get_agent_download_info TO anon, authenticated;