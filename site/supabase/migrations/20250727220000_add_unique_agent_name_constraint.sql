-- Add unique constraint to agent name field to prevent duplicate agent names
-- This migration ensures data integrity at the database level

-- First, let's identify any existing duplicate agent names that would prevent the constraint
-- This query can be used to check for duplicates before applying the migration:
-- SELECT name, COUNT(*) 
-- FROM public.agents 
-- GROUP BY name 
-- HAVING COUNT(*) > 1;

-- Add unique constraint on the agent name field
-- This will prevent any future attempts to insert agents with duplicate names
ALTER TABLE public.agents 
ADD CONSTRAINT unique_agent_name UNIQUE (name);

-- Create a partial index for better performance on name lookups
-- This index will help with both uniqueness checks and name-based queries
CREATE INDEX IF NOT EXISTS idx_agents_name_unique 
ON public.agents(name) 
WHERE name IS NOT NULL;

-- Update the existing function to use the database constraint instead of manual checking
-- The function will now rely on the database constraint to prevent duplicates
-- and provide better error handling
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
  
  -- Validate agent name format (only allow alphanumeric, hyphens, underscores)
  IF NOT agent_name ~ '^[a-zA-Z0-9_-]+$' THEN
    RETURN jsonb_build_object(
      'success', false,
      'error', 'Agent name can only contain letters, numbers, hyphens, and underscores'
    );
  END IF;
  
  -- Get user profile for author name fallback
  SELECT display_name, github_username INTO profile_record
  FROM public.profiles 
  WHERE user_id = user_id;
  
  -- Attempt to create the agent - let the unique constraint handle duplicates
  BEGIN
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
    
  EXCEPTION
    WHEN unique_violation THEN
      -- Handle the unique constraint violation specifically
      IF SQLERRM LIKE '%unique_agent_name%' THEN
        RETURN jsonb_build_object(
          'success', false,
          'error', 'Agent name already exists. Please choose a different name.',
          'error_code', 'DUPLICATE_AGENT_NAME'
        );
      ELSE
        RETURN jsonb_build_object(
          'success', false,
          'error', 'A unique constraint violation occurred',
          'error_code', 'UNIQUE_VIOLATION'
        );
      END IF;
    WHEN OTHERS THEN
      RETURN jsonb_build_object(
        'success', false,
        'error', 'An unexpected error occurred: ' || SQLERRM,
        'error_code', 'UNEXPECTED_ERROR'
      );
  END;
END;
$$;

-- Update function permissions
GRANT EXECUTE ON FUNCTION public.create_agent TO authenticated;