-- Fix get_agent_download_info function signature
-- Drop any existing versions of the function that might have different parameter signatures
DROP FUNCTION IF EXISTS public.get_agent_download_info(TEXT, UUID, TEXT);
DROP FUNCTION IF EXISTS public.get_agent_download_info(TEXT, TEXT, UUID);
DROP FUNCTION IF EXISTS public.get_agent_download_info(p_agent_name TEXT, p_user_id UUID, p_version_text TEXT);
DROP FUNCTION IF EXISTS public.get_agent_download_info(p_agent_name TEXT, p_version_text TEXT, p_user_id UUID);
DROP FUNCTION IF EXISTS public.get_agent_download_info(TEXT, TEXT);
DROP FUNCTION IF EXISTS public.get_agent_download_info(p_agent_name TEXT, p_version_text TEXT);

-- Recreate the function with the correct signature
CREATE OR REPLACE FUNCTION public.get_agent_download_info(
  p_agent_name TEXT,
  p_version_text TEXT DEFAULT ''
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
DECLARE
  agent_record RECORD;
  version_record RECORD;
  package_record RECORD;
BEGIN
  -- Find the agent
  SELECT id, name INTO agent_record
  FROM public.agents 
  WHERE name = p_agent_name AND is_public = true;
  
  IF NOT FOUND THEN
    RETURN;
  END IF;
  
  -- Find the version (use latest if not specified or "latest")
  IF p_version_text = '' OR p_version_text = 'latest' THEN
    -- Get the latest version
    SELECT av.id, av.version, av.checksum, av.package_size INTO version_record
    FROM public.agent_versions av
    WHERE av.agent_id = agent_record.id
      AND av.yanked = false
    ORDER BY av.created_at DESC
    LIMIT 1;
  ELSE
    -- Get specific version
    SELECT av.id, av.version, av.checksum, av.package_size INTO version_record
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
    agent_record.name::TEXT,
    version_record.version::TEXT,
    package_record.file_path::TEXT,
    COALESCE(package_record.checksum, version_record.checksum, '')::TEXT,
    COALESCE(package_record.file_size, version_record.package_size, 0)::BIGINT;
END;
$$;

-- Grant execute permissions
GRANT EXECUTE ON FUNCTION public.get_agent_download_info TO anon, authenticated;

-- Create indexes to optimize queries (if they don't already exist)
CREATE INDEX IF NOT EXISTS idx_agent_versions_latest 
ON public.agent_versions(agent_id, created_at DESC) 
WHERE yanked = false;

CREATE INDEX IF NOT EXISTS idx_agent_packages_completed 
ON public.agent_packages(version_id, created_at DESC) 
WHERE upload_completed = true;