-- Verify and fix agents table schema inconsistencies
-- This migration ensures the agents table has all required columns after the migration rollbacks

-- First, let's check the current state and add any missing columns
DO $$
BEGIN
    -- Ensure current_version column exists and has proper defaults
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'agents' AND column_name = 'current_version' AND table_schema = 'public') THEN
        ALTER TABLE public.agents ADD COLUMN current_version TEXT DEFAULT '1.0.0';
        UPDATE public.agents SET current_version = '1.0.0' WHERE current_version IS NULL;
        ALTER TABLE public.agents ALTER COLUMN current_version SET NOT NULL;
        RAISE NOTICE 'Added current_version column to agents table';
    END IF;
    
    -- Ensure author_name column exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'agents' AND column_name = 'author_name' AND table_schema = 'public') THEN
        ALTER TABLE public.agents ADD COLUMN author_name TEXT;
        RAISE NOTICE 'Added author_name column to agents table';
    END IF;
    
    -- Ensure download_count column exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'agents' AND column_name = 'download_count' AND table_schema = 'public') THEN
        ALTER TABLE public.agents ADD COLUMN download_count BIGINT DEFAULT 0;
        UPDATE public.agents SET download_count = 0 WHERE download_count IS NULL;
        RAISE NOTICE 'Added download_count column to agents table';
    END IF;
    
    -- Ensure other package management columns exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'agents' AND column_name = 'license' AND table_schema = 'public') THEN
        ALTER TABLE public.agents ADD COLUMN license TEXT;
        RAISE NOTICE 'Added license column to agents table';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'agents' AND column_name = 'homepage' AND table_schema = 'public') THEN
        ALTER TABLE public.agents ADD COLUMN homepage TEXT;
        RAISE NOTICE 'Added homepage column to agents table';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'agents' AND column_name = 'repository' AND table_schema = 'public') THEN
        ALTER TABLE public.agents ADD COLUMN repository TEXT;
        RAISE NOTICE 'Added repository column to agents table';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'agents' AND column_name = 'keywords' AND table_schema = 'public') THEN
        ALTER TABLE public.agents ADD COLUMN keywords TEXT[];
        RAISE NOTICE 'Added keywords column to agents table';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'agents' AND column_name = 'readme' AND table_schema = 'public') THEN
        ALTER TABLE public.agents ADD COLUMN readme TEXT;
        RAISE NOTICE 'Added readme column to agents table';
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'agents' AND column_name = 'latest_version_id' AND table_schema = 'public') THEN
        ALTER TABLE public.agents ADD COLUMN latest_version_id UUID;
        RAISE NOTICE 'Added latest_version_id column to agents table';
    END IF;
END $$;

-- Verify that RLS is enabled on the agents table
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_tables 
        WHERE tablename = 'agents' 
        AND schemaname = 'public' 
        AND rowsecurity = true
    ) THEN
        ALTER TABLE public.agents ENABLE ROW LEVEL SECURITY;
        RAISE NOTICE 'Enabled RLS on agents table';
    END IF;
END $$;

-- Ensure the basic RLS policies exist for agents table
-- First drop existing policies to avoid conflicts, then recreate them
DROP POLICY IF EXISTS "Public agents are viewable by everyone" ON public.agents;
DROP POLICY IF EXISTS "Users can create their own agents" ON public.agents;
DROP POLICY IF EXISTS "Users can update their own agents" ON public.agents;
DROP POLICY IF EXISTS "Users can delete their own agents" ON public.agents;

-- Recreate the essential policies
CREATE POLICY "Public agents are viewable by everyone" 
ON public.agents 
FOR SELECT 
USING (is_public = true OR auth.uid() = user_id);

CREATE POLICY "Users can create their own agents" 
ON public.agents 
FOR INSERT 
WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own agents" 
ON public.agents 
FOR UPDATE 
USING (auth.uid() = user_id);

CREATE POLICY "Users can delete their own agents" 
ON public.agents 
FOR DELETE 
USING (auth.uid() = user_id);

-- Add indexes for performance if they don't exist
CREATE INDEX IF NOT EXISTS idx_agents_user_id ON public.agents(user_id);
CREATE INDEX IF NOT EXISTS idx_agents_created_at ON public.agents(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agents_view_count ON public.agents(view_count DESC);
CREATE INDEX IF NOT EXISTS idx_agents_is_public ON public.agents(is_public);

-- Create a function to test agent fetching (for debugging)
CREATE OR REPLACE FUNCTION public.test_agent_fetching(test_user_id UUID)
RETURNS TABLE (
    agent_count INTEGER,
    sample_agent_id UUID,
    sample_agent_name TEXT,
    error_message TEXT
)
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
    BEGIN
        SELECT COUNT(*)::INTEGER, 
               (SELECT id FROM public.agents WHERE user_id = test_user_id LIMIT 1),
               (SELECT name FROM public.agents WHERE user_id = test_user_id LIMIT 1),
               NULL::TEXT
        INTO agent_count, sample_agent_id, sample_agent_name, error_message
        FROM public.agents 
        WHERE user_id = test_user_id;
        
        RETURN NEXT;
    EXCEPTION WHEN OTHERS THEN
        agent_count := -1;
        sample_agent_id := NULL;
        sample_agent_name := NULL;
        error_message := SQLERRM;
        RETURN NEXT;
    END;
END;
$$;

-- Grant execute permission on the test function
GRANT EXECUTE ON FUNCTION public.test_agent_fetching TO authenticated;

-- Log completion
DO $$
BEGIN
    RAISE NOTICE 'Schema verification and repair completed successfully';
END $$;