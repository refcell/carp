-- Fix the security issues with function search paths
-- Use CREATE OR REPLACE instead of DROP to avoid dependency issues

-- Recreate functions with proper security settings
CREATE OR REPLACE FUNCTION public.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER SET search_path = '';

-- Create function to handle new user profile creation with proper security
CREATE OR REPLACE FUNCTION public.handle_new_user()
RETURNS TRIGGER AS $$
BEGIN
  INSERT INTO public.profiles (user_id, github_username, display_name, avatar_url)
  VALUES (
    NEW.id,
    NEW.raw_user_meta_data ->> 'user_name',
    NEW.raw_user_meta_data ->> 'full_name',
    NEW.raw_user_meta_data ->> 'avatar_url'
  );
  RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER SET search_path = '';