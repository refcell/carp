-- Enhance existing API keys table with improved security and functionality
-- This migration builds upon the existing api_keys table (20250729100000_add_api_keys_table.sql)

-- Add missing columns and constraints for better security
ALTER TABLE public.api_keys 
  ADD COLUMN IF NOT EXISTS key_prefix TEXT;

-- Update the existing prefix column to be key_prefix if it doesn't exist
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                 WHERE table_name = 'api_keys' AND column_name = 'key_prefix' AND table_schema = 'public') THEN
    -- If key_prefix doesn't exist but prefix does, rename prefix to key_prefix
    IF EXISTS (SELECT 1 FROM information_schema.columns 
               WHERE table_name = 'api_keys' AND column_name = 'prefix' AND table_schema = 'public') THEN
      ALTER TABLE public.api_keys RENAME COLUMN prefix TO key_prefix;
      RAISE NOTICE 'Renamed prefix column to key_prefix';
    ELSE
      -- Add key_prefix column if neither exists
      ALTER TABLE public.api_keys ADD COLUMN key_prefix TEXT;
      RAISE NOTICE 'Added key_prefix column';
    END IF;
  END IF;
END $$;

-- Make name column nullable (users should be able to create unnamed keys)
ALTER TABLE public.api_keys ALTER COLUMN name DROP NOT NULL;

-- Add constraints for better security and uniqueness
DO $$
BEGIN
  -- Add unique constraint on key_prefix
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'unique_api_key_prefix') THEN
    ALTER TABLE public.api_keys ADD CONSTRAINT unique_api_key_prefix UNIQUE (key_prefix);
    RAISE NOTICE 'Added unique constraint on key_prefix';
  END IF;
  
  -- Add unique constraint for user + name combination (allowing NULLs)
  IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'unique_user_key_name') THEN
    ALTER TABLE public.api_keys ADD CONSTRAINT unique_user_key_name UNIQUE (user_id, name) DEFERRABLE INITIALLY DEFERRED;
    RAISE NOTICE 'Added unique constraint on user_id + name';
  END IF;
END $$;

-- Create partial unique index that allows multiple NULL names per user
CREATE UNIQUE INDEX IF NOT EXISTS idx_api_keys_user_name_not_null 
ON public.api_keys (user_id, name) 
WHERE name IS NOT NULL;

-- Add additional indexes for better performance
CREATE INDEX IF NOT EXISTS idx_api_keys_created_at ON public.api_keys(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_api_keys_last_used_at ON public.api_keys(last_used_at DESC) WHERE last_used_at IS NOT NULL;

-- Enhance the validate_api_key function with better naming and functionality
DROP FUNCTION IF EXISTS public.validate_api_key(TEXT);

CREATE OR REPLACE FUNCTION public.verify_api_key(key_hash_param TEXT)
RETURNS TABLE (
  user_id UUID, 
  key_id UUID, 
  scopes TEXT[],
  is_valid BOOLEAN
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    ak.user_id,
    ak.id as key_id,
    ak.scopes,
    (ak.is_active = true AND (ak.expires_at IS NULL OR ak.expires_at > now())) as is_valid
  FROM public.api_keys ak
  WHERE ak.key_hash = key_hash_param;
  
  -- If no key found, return a row with null values and is_valid = false
  IF NOT FOUND THEN
    RETURN QUERY SELECT NULL::UUID, NULL::UUID, NULL::TEXT[], false;
  END IF;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Enhance the update_api_key_last_used function to return success status
DROP FUNCTION IF EXISTS public.update_api_key_last_used(TEXT);

CREATE OR REPLACE FUNCTION public.update_api_key_last_used(key_hash_param TEXT)
RETURNS BOOLEAN AS $$
DECLARE
  rows_updated INTEGER;
BEGIN
  UPDATE public.api_keys 
  SET last_used_at = now(), updated_at = now()
  WHERE key_hash = key_hash_param 
    AND is_active = true 
    AND (expires_at IS NULL OR expires_at > now());
  
  GET DIAGNOSTICS rows_updated = ROW_COUNT;
  RETURN (rows_updated > 0);
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Add function to validate API key format and business rules
CREATE OR REPLACE FUNCTION public.validate_api_key_format()
RETURNS TRIGGER AS $$
BEGIN
  -- Ensure key_hash is provided and looks like a valid hash
  IF NEW.key_hash IS NULL OR length(NEW.key_hash) < 64 THEN
    RAISE EXCEPTION 'API key hash must be a valid SHA-256 hash (64+ characters)';
  END IF;
  
  -- Ensure key_prefix is provided and has reasonable length
  IF NEW.key_prefix IS NULL OR length(NEW.key_prefix) < 8 OR length(NEW.key_prefix) > 20 THEN
    RAISE EXCEPTION 'API key prefix must be between 8 and 20 characters';
  END IF;
  
  -- Ensure key_prefix starts with expected format
  IF NOT NEW.key_prefix ~ '^carp_[a-zA-Z0-9_]+$' THEN
    RAISE EXCEPTION 'API key prefix must start with "carp_" followed by alphanumeric characters and underscores';
  END IF;
  
  -- If expiration is set, ensure it's in the future
  IF NEW.expires_at IS NOT NULL AND NEW.expires_at <= now() THEN
    RAISE EXCEPTION 'API key expiration date must be in the future';
  END IF;
  
  -- Limit the number of active keys per user (prevent abuse)
  IF NEW.is_active = true THEN
    DECLARE
      active_key_count INTEGER;
    BEGIN
      SELECT COUNT(*) INTO active_key_count 
      FROM public.api_keys 
      WHERE user_id = NEW.user_id AND is_active = true AND id != COALESCE(NEW.id, gen_random_uuid());
      
      IF active_key_count >= 10 THEN
        RAISE EXCEPTION 'Users cannot have more than 10 active API keys';
      END IF;
    END;
  END IF;
  
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for API key validation
DROP TRIGGER IF EXISTS validate_api_key_before_insert_or_update ON public.api_keys;
CREATE TRIGGER validate_api_key_before_insert_or_update
  BEFORE INSERT OR UPDATE ON public.api_keys
  FOR EACH ROW
  EXECUTE FUNCTION public.validate_api_key_format();

-- Add function to automatically mark expired keys as inactive
CREATE OR REPLACE FUNCTION public.deactivate_expired_api_keys()
RETURNS INTEGER AS $$
DECLARE
  deactivated_count INTEGER;
BEGIN
  UPDATE public.api_keys 
  SET is_active = false, updated_at = now()
  WHERE is_active = true 
    AND expires_at IS NOT NULL 
    AND expires_at <= now();
  
  GET DIAGNOSTICS deactivated_count = ROW_COUNT;
  RETURN deactivated_count;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Improve the generate_api_key function for better security and format
DROP FUNCTION IF EXISTS public.generate_api_key();

CREATE OR REPLACE FUNCTION public.generate_api_key_secure()
RETURNS TABLE(
  full_key TEXT,
  key_hash TEXT,
  key_prefix TEXT
) AS $$
DECLARE
  random_part TEXT;
  generated_key TEXT;
  hash_result TEXT;
  prefix_result TEXT;
BEGIN
  -- Generate 32 random characters (more secure than the original 24)
  random_part := encode(gen_random_bytes(24), 'base64');
  random_part := translate(random_part, '+/=', 'Aa0');
  random_part := substring(random_part, 1, 32);
  
  -- Create the full key with consistent format
  generated_key := 'carp_k_' || random_part;
  
  -- Generate SHA-256 hash (using digest function if available, otherwise note for application-level hashing)
  -- Note: PostgreSQL's digest function requires pgcrypto extension
  -- In production, this should be done in the application layer with proper crypto libraries
  hash_result := 'HASH_IN_APPLICATION_' || generated_key;  -- Placeholder - hash in application
  
  -- Extract prefix for identification
  prefix_result := substring(generated_key, 1, 16);
  
  full_key := generated_key;
  key_hash := hash_result;
  key_prefix := prefix_result;
  
  RETURN NEXT;
END;
$$ LANGUAGE plpgsql;

-- Grant execute permissions on the new functions
GRANT EXECUTE ON FUNCTION public.verify_api_key TO authenticated;
GRANT EXECUTE ON FUNCTION public.update_api_key_last_used TO authenticated;
GRANT EXECUTE ON FUNCTION public.deactivate_expired_api_keys TO authenticated;
GRANT EXECUTE ON FUNCTION public.generate_api_key_secure TO authenticated;

-- Update existing data to ensure key_prefix is populated if missing
DO $$
BEGIN
  -- If there are existing records without key_prefix, we need to update them
  -- This is a one-time data migration for existing records
  UPDATE public.api_keys 
  SET key_prefix = 'carp_legacy_' || substring(md5(random()::text), 1, 8)
  WHERE key_prefix IS NULL OR key_prefix = '';
  
  -- Now make key_prefix NOT NULL since all records should have it
  ALTER TABLE public.api_keys ALTER COLUMN key_prefix SET NOT NULL;
END $$;

-- Add helpful comments for documentation
COMMENT ON TABLE public.api_keys IS 'Enhanced API keys table with improved security. Keys are hashed using SHA-256 and never stored in plaintext.';
COMMENT ON COLUMN public.api_keys.key_hash IS 'SHA-256 hash of the full API key. Used for authentication.';
COMMENT ON COLUMN public.api_keys.key_prefix IS 'First 16 characters of the API key for identification in UI. Format: carp_k_xxxxxxxx';
COMMENT ON COLUMN public.api_keys.name IS 'Optional human-readable name/description for the key. NULL is allowed for unnamed keys.';
COMMENT ON COLUMN public.api_keys.scopes IS 'Array of permission scopes for fine-grained access control';
COMMENT ON COLUMN public.api_keys.last_used_at IS 'Timestamp of when this key was last used for authentication';
COMMENT ON COLUMN public.api_keys.expires_at IS 'Optional expiration timestamp. NULL means no expiration.';
COMMENT ON COLUMN public.api_keys.is_active IS 'Whether the key is active. Inactive keys cannot be used for authentication.';

-- Log successful completion
DO $$
BEGIN
  RAISE NOTICE 'API keys table enhanced successfully';
  RAISE NOTICE 'Key improvements: nullable names, better validation, enhanced functions, additional indexes';
  RAISE NOTICE 'Remember to implement SHA-256 hashing in your application code';
  RAISE NOTICE 'New API key format: carp_k_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx (carp_k_ prefix + 32 random chars)';
END $$;