-- Fix agents table RLS policies to support API key authentication
-- 
-- PROBLEM: The current RLS policy only works with JWT authentication (auth.uid())
-- API key authentication doesn't set auth.uid(), causing RLS policy violations
--
-- SOLUTION: Create a function to get the current user ID from either:
-- 1. JWT authentication (auth.uid())  
-- 2. Service role context with explicit user_id (for API key auth)

-- Create function to get current user ID for RLS policies
-- This handles both JWT authentication and service role API key authentication
CREATE OR REPLACE FUNCTION public.get_current_user_id()
RETURNS UUID AS $$
BEGIN
  -- First try auth.uid() for JWT authentication
  IF auth.uid() IS NOT NULL THEN
    RETURN auth.uid();
  END IF;
  
  -- For service role requests (API key auth), return NULL
  -- The service role will bypass RLS entirely
  RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Drop existing agents RLS policies
DROP POLICY IF EXISTS "Users can create their own agents" ON public.agents;
DROP POLICY IF EXISTS "Users can update their own agents" ON public.agents;
DROP POLICY IF EXISTS "Users can delete their own agents" ON public.agents;

-- Create new RLS policies that work with both JWT and API key authentication
CREATE POLICY "Users can create their own agents" 
ON public.agents 
FOR INSERT 
WITH CHECK (
  -- Allow if authenticated via JWT and user owns the agent
  (auth.uid() IS NOT NULL AND auth.uid() = user_id)
  OR
  -- Allow if using service role (for API key authentication)
  -- The application layer has already validated the API key
  (auth.role() = 'service_role')
);

CREATE POLICY "Users can update their own agents" 
ON public.agents 
FOR UPDATE 
USING (
  -- Allow if authenticated via JWT and user owns the agent
  (auth.uid() IS NOT NULL AND auth.uid() = user_id)
  OR
  -- Allow if using service role (for API key authentication)
  (auth.role() = 'service_role')
);

CREATE POLICY "Users can delete their own agents" 
ON public.agents 
FOR DELETE 
USING (
  -- Allow if authenticated via JWT and user owns the agent
  (auth.uid() IS NOT NULL AND auth.uid() = user_id)
  OR
  -- Allow if using service role (for API key authentication)
  (auth.role() = 'service_role')
);

-- Keep the existing SELECT policy unchanged (it already works correctly)
-- CREATE POLICY "Public agents are viewable by everyone" 
-- ON public.agents 
-- FOR SELECT 
-- USING (is_public = true OR auth.uid() = user_id);

-- Add comment explaining the solution
COMMENT ON FUNCTION public.get_current_user_id() IS 
'Returns the current user ID for RLS policies. Supports both JWT authentication (auth.uid()) and service role API key authentication.';

COMMENT ON POLICY "Users can create their own agents" ON public.agents IS 
'Allows agent creation via JWT authentication or service role (API key authentication). The application layer validates API key ownership.';

COMMENT ON POLICY "Users can update their own agents" ON public.agents IS 
'Allows agent updates via JWT authentication or service role (API key authentication). The application layer validates API key ownership.';

COMMENT ON POLICY "Users can delete their own agents" ON public.agents IS 
'Allows agent deletion via JWT authentication or service role (API key authentication). The application layer validates API key ownership.';