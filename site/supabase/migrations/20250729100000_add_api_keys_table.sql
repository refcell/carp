-- Create API keys table for authentication
CREATE TABLE public.api_keys (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  name TEXT NOT NULL, -- User-friendly name for the API key
  key_hash TEXT NOT NULL UNIQUE, -- Hashed API key for security
  prefix TEXT NOT NULL, -- First 8 characters of the original key for identification
  scopes TEXT[] NOT NULL DEFAULT '{}', -- Array of permissions/scopes
  is_active BOOLEAN NOT NULL DEFAULT true,
  last_used_at TIMESTAMP WITH TIME ZONE,
  expires_at TIMESTAMP WITH TIME ZONE, -- NULL means no expiration
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

-- Enable Row Level Security
ALTER TABLE public.api_keys ENABLE ROW LEVEL SECURITY;

-- API keys policies
CREATE POLICY "Users can view their own API keys" 
ON public.api_keys 
FOR SELECT 
USING (auth.uid() = user_id);

CREATE POLICY "Users can create their own API keys" 
ON public.api_keys 
FOR INSERT 
WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own API keys" 
ON public.api_keys 
FOR UPDATE 
USING (auth.uid() = user_id);

CREATE POLICY "Users can delete their own API keys" 
ON public.api_keys 
FOR DELETE 
USING (auth.uid() = user_id);

-- Create indexes for better performance
CREATE INDEX idx_api_keys_user_id ON public.api_keys(user_id);
CREATE INDEX idx_api_keys_key_hash ON public.api_keys(key_hash);
CREATE INDEX idx_api_keys_prefix ON public.api_keys(prefix);
CREATE INDEX idx_api_keys_is_active ON public.api_keys(is_active);
CREATE INDEX idx_api_keys_expires_at ON public.api_keys(expires_at);

-- Create trigger for automatic timestamp updates
CREATE TRIGGER update_api_keys_updated_at
  BEFORE UPDATE ON public.api_keys
  FOR EACH ROW
  EXECUTE FUNCTION public.update_updated_at_column();

-- Create function to validate API key and return user info
-- Uses SHA-256 hash matching for API key validation
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

-- Create function to update last_used_at for API key
CREATE OR REPLACE FUNCTION public.update_api_key_last_used(api_key_hash TEXT)
RETURNS VOID AS $$
BEGIN
  UPDATE public.api_keys 
  SET last_used_at = now()
  WHERE key_hash = api_key_hash;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Create function to generate API key with proper format
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