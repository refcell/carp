-- Rollback migration for API keys enhancements
-- This file reverts the enhancements made in 20250729185700_enhance_api_keys_table.sql
-- Note: This does NOT remove the original api_keys table, only the enhancements

-- Drop enhanced triggers first (to avoid dependency issues)
DROP TRIGGER IF EXISTS validate_api_key_before_insert_or_update ON public.api_keys;

-- Drop enhanced functions
DROP FUNCTION IF EXISTS public.verify_api_key(TEXT);
DROP FUNCTION IF EXISTS public.update_api_key_last_used(TEXT);
DROP FUNCTION IF EXISTS public.deactivate_expired_api_keys();
DROP FUNCTION IF EXISTS public.validate_api_key_format();
DROP FUNCTION IF EXISTS public.generate_api_key_secure();

-- Drop enhanced indexes
DROP INDEX IF EXISTS idx_api_keys_user_name_not_null;
DROP INDEX IF EXISTS idx_api_keys_created_at;
DROP INDEX IF EXISTS idx_api_keys_last_used_at;

-- Drop enhanced constraints (be careful with this - check if data exists)
ALTER TABLE public.api_keys DROP CONSTRAINT IF EXISTS unique_api_key_prefix;
ALTER TABLE public.api_keys DROP CONSTRAINT IF EXISTS unique_user_key_name;

-- Revert column changes - make name NOT NULL again if it was originally NOT NULL
-- (Check the original migration to see the intended state)
DO $$
BEGIN
  -- Note: The original migration had name as NOT NULL, so we revert that
  -- But be careful - if there are NULL names in the data, this will fail
  BEGIN
    ALTER TABLE public.api_keys ALTER COLUMN name SET NOT NULL;
  EXCEPTION WHEN OTHERS THEN
    RAISE NOTICE 'Could not set name column to NOT NULL - there may be NULL values. Original constraint may have been different.';
  END;
END $$;

-- Restore original functions (from 20250729100000_add_api_keys_table.sql)
CREATE OR REPLACE FUNCTION public.validate_api_key(api_key_hash TEXT)
RETURNS TABLE(
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
    (ak.is_active AND (ak.expires_at IS NULL OR ak.expires_at > now())) as is_valid
  FROM public.api_keys ak
  WHERE ak.key_hash = api_key_hash
  LIMIT 1;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION public.update_api_key_last_used(api_key_hash TEXT)
RETURNS VOID AS $$
BEGIN
  UPDATE public.api_keys 
  SET last_used_at = now()
  WHERE key_hash = api_key_hash;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE OR REPLACE FUNCTION public.generate_api_key()
RETURNS TEXT AS $$
DECLARE
  key_part1 TEXT;
  key_part2 TEXT;
  key_part3 TEXT;
BEGIN
  -- Generate three parts of 8 characters each (24 total + 2 separators = 26 chars)
  key_part1 := encode(gen_random_bytes(6), 'base64');
  key_part1 := translate(key_part1, '+/=', 'Aa0');
  key_part1 := substring(key_part1, 1, 8);
  
  key_part2 := encode(gen_random_bytes(6), 'base64');
  key_part2 := translate(key_part2, '+/=', 'Bb1');
  key_part2 := substring(key_part2, 1, 8);
  
  key_part3 := encode(gen_random_bytes(6), 'base64');
  key_part3 := translate(key_part3, '+/=', 'Cc2');
  key_part3 := substring(key_part3, 1, 8);
  
  RETURN 'carp_' || key_part1 || '_' || key_part2 || '_' || key_part3;
END;
$$ LANGUAGE plpgsql;

-- Remove comments added by the enhancement
COMMENT ON TABLE public.api_keys IS NULL;
COMMENT ON COLUMN public.api_keys.key_hash IS NULL;
COMMENT ON COLUMN public.api_keys.key_prefix IS NULL;
COMMENT ON COLUMN public.api_keys.name IS NULL;
COMMENT ON COLUMN public.api_keys.scopes IS NULL;
COMMENT ON COLUMN public.api_keys.last_used_at IS NULL;
COMMENT ON COLUMN public.api_keys.expires_at IS NULL;
COMMENT ON COLUMN public.api_keys.is_active IS NULL;

-- Log successful rollback
DO $$
BEGIN
  RAISE NOTICE 'API keys enhancements have been rolled back to the original state';
  RAISE NOTICE 'The base api_keys table remains intact from the original migration';
END $$;