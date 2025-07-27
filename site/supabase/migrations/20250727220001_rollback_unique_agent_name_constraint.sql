-- Rollback migration for unique agent name constraint
-- This migration removes the unique constraint and restores the original function
-- Use this if you need to rollback the unique constraint changes

-- Remove the unique constraint
ALTER TABLE public.agents 
DROP CONSTRAINT IF EXISTS unique_agent_name;

-- Remove the performance index
DROP INDEX IF EXISTS idx_agents_name_unique;

-- Restore the original create_agent function with manual duplicate checking
CREATE OR REPLACE FUNCTION public.create_agent(
  agent_name TEXT,
  description TEXT,
  author_name TEXT DEFAULT '',
  tags TEXT[] DEFAULT '{}',
  keywords TEXT[] DEFAULT '{}',
  license TEXT DEFAULT '',
  homepage TEXT DEFAULT '',
  repository TEXT DEFAULT '',
  readme TEXT DEFAULT '',
  is_public BOOLEAN DEFAULT true
)
RETURNS jsonb
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = ''
AS $$
DECLARE
  user_id UUID;
  agent_id UUID;
  profile_record RECORD;
BEGIN
  -- Get the authenticated user ID
  user_id := auth.uid();
  
  IF user_id IS NULL THEN
    RETURN jsonb_build_object(
      'success', false,
      'error', 'User not authenticated'
    );
  END IF;
  
  -- Validate agent name
  IF agent_name IS NULL OR LENGTH(TRIM(agent_name)) = 0 THEN
    RETURN jsonb_build_object(
      'success', false,
      'error', 'Agent name cannot be empty'
    );
  END IF;
  
  -- Get user profile for author name fallback
  SELECT display_name, github_username INTO profile_record
  FROM public.profiles 
  WHERE user_id = user_id;
  
  -- Check if agent name is already taken (manual checking)
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

-- Update function permissions
GRANT EXECUTE ON FUNCTION public.create_agent TO authenticated;