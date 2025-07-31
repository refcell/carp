-- Fix API key user synchronization
--
-- PROBLEM: API keys reference user_id but users may not exist in auth.users
-- The sync_jwt_user function tries to insert into non-existent 'users' table
--
-- SOLUTION: 
-- 1. Fix the sync function to use the correct table (profiles)
-- 2. Create a proper user sync function for API key users
-- 3. Handle the case where API key users don't have auth.users entries

-- Create function to sync API key users into the profiles table
-- This ensures users referenced by API keys have corresponding profile entries
CREATE OR REPLACE FUNCTION public.sync_api_key_user(
  user_uuid UUID, 
  user_email TEXT DEFAULT NULL, 
  github_username TEXT DEFAULT NULL
)
RETURNS BOOLEAN AS $$
DECLARE
  profile_exists BOOLEAN;
  auth_user_exists BOOLEAN;
BEGIN
  -- Check if user exists in auth.users
  SELECT EXISTS(
    SELECT 1 FROM auth.users WHERE id = user_uuid
  ) INTO auth_user_exists;
  
  -- Check if profile exists
  SELECT EXISTS(
    SELECT 1 FROM public.profiles WHERE user_id = user_uuid
  ) INTO profile_exists;
  
  -- If auth user doesn't exist, we have a problem
  -- API keys should only be created for users who went through proper auth
  IF NOT auth_user_exists THEN
    RAISE NOTICE 'API key references non-existent auth user: %', user_uuid;
    -- For now, we'll allow this but log it
    -- In production, this should be fixed by ensuring proper user creation flow
  END IF;
  
  -- Ensure profile exists (this is safe to upsert)
  IF NOT profile_exists AND auth_user_exists THEN
    INSERT INTO public.profiles (
      user_id, 
      github_username,
      display_name,
      created_at,
      updated_at
    ) VALUES (
      user_uuid,
      github_username,
      COALESCE(github_username, 'API User'),
      now(),
      now()
    ) ON CONFLICT (user_id) DO UPDATE SET
      github_username = COALESCE(EXCLUDED.github_username, profiles.github_username),
      updated_at = now();
  END IF;
  
  RETURN auth_user_exists;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Create a fixed version of the user sync for JWT users
-- This fixes the original sync_jwt_user function which tried to insert into 'users' table
CREATE OR REPLACE FUNCTION public.sync_jwt_user_fixed(
  user_uuid UUID, 
  user_email TEXT DEFAULT NULL, 
  github_username TEXT DEFAULT NULL,
  display_name TEXT DEFAULT NULL,
  avatar_url TEXT DEFAULT NULL
)
RETURNS BOOLEAN AS $$
DECLARE
  profile_exists BOOLEAN;
  auth_user_exists BOOLEAN;
BEGIN
  -- Check if user exists in auth.users (should always be true for JWT users)
  SELECT EXISTS(
    SELECT 1 FROM auth.users WHERE id = user_uuid
  ) INTO auth_user_exists;
  
  -- Check if profile exists
  SELECT EXISTS(
    SELECT 1 FROM public.profiles WHERE user_id = user_uuid
  ) INTO profile_exists;
  
  -- Ensure profile exists
  IF NOT profile_exists AND auth_user_exists THEN
    INSERT INTO public.profiles (
      user_id, 
      github_username,
      display_name,
      avatar_url,
      created_at,
      updated_at
    ) VALUES (
      user_uuid,
      github_username,
      display_name,
      avatar_url,
      now(),
      now()
    ) ON CONFLICT (user_id) DO UPDATE SET
      github_username = COALESCE(EXCLUDED.github_username, profiles.github_username),
      display_name = COALESCE(EXCLUDED.display_name, profiles.display_name),
      avatar_url = COALESCE(EXCLUDED.avatar_url, profiles.avatar_url),
      updated_at = now();
  END IF;
  
  RETURN auth_user_exists;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Add comments explaining the functions
COMMENT ON FUNCTION public.sync_api_key_user(UUID, TEXT, TEXT) IS 
'Ensures API key users have corresponding profile entries. Handles the case where API key user_id may not exist in auth.users.';

COMMENT ON FUNCTION public.sync_jwt_user_fixed(UUID, TEXT, TEXT, TEXT, TEXT) IS 
'Fixed version of user sync for JWT authentication. Properly inserts into profiles table instead of non-existent users table.';