-- Recreate package management schema after remote sync
-- This migration restores all the tables and functionality that were removed

-- First, extend the existing agents table with package management fields
ALTER TABLE public.agents 
  ADD COLUMN IF NOT EXISTS current_version TEXT DEFAULT '1.0.0',
  ADD COLUMN IF NOT EXISTS author_name TEXT,
  ADD COLUMN IF NOT EXISTS license TEXT,
  ADD COLUMN IF NOT EXISTS homepage TEXT,
  ADD COLUMN IF NOT EXISTS repository TEXT,
  ADD COLUMN IF NOT EXISTS keywords TEXT[],
  ADD COLUMN IF NOT EXISTS download_count BIGINT DEFAULT 0,
  ADD COLUMN IF NOT EXISTS latest_version_id UUID,
  ADD COLUMN IF NOT EXISTS readme TEXT;

-- Update existing agents to have default values
UPDATE public.agents 
SET 
  current_version = '1.0.0',
  download_count = 0
WHERE current_version IS NULL;

-- Make current_version NOT NULL after setting defaults
ALTER TABLE public.agents 
  ALTER COLUMN current_version SET NOT NULL;

-- Create agent_versions table to track all versions of each agent
CREATE TABLE IF NOT EXISTS public.agent_versions (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  agent_id UUID NOT NULL REFERENCES public.agents(id) ON DELETE CASCADE,
  version TEXT NOT NULL,
  description TEXT,
  changelog TEXT,
  definition JSONB NOT NULL,
  package_size BIGINT,
  checksum TEXT,
  download_count BIGINT DEFAULT 0,
  is_pre_release BOOLEAN DEFAULT false,
  yanked BOOLEAN DEFAULT false,
  yanked_reason TEXT,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  UNIQUE(agent_id, version)
);

-- Create agent_packages table for storing package file metadata and URLs
CREATE TABLE IF NOT EXISTS public.agent_packages (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  version_id UUID NOT NULL REFERENCES public.agent_versions(id) ON DELETE CASCADE,
  file_name TEXT NOT NULL,
  file_path TEXT NOT NULL,
  content_type TEXT NOT NULL DEFAULT 'application/gzip',
  file_size BIGINT NOT NULL,
  checksum TEXT NOT NULL,
  upload_completed BOOLEAN DEFAULT false,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  UNIQUE(version_id, file_name)
);

-- Create API tokens table for CLI authentication
CREATE TABLE IF NOT EXISTS public.api_tokens (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  token_name TEXT NOT NULL,
  token_hash TEXT NOT NULL UNIQUE,
  token_prefix TEXT NOT NULL, -- First 8 chars for identification
  scopes TEXT[] DEFAULT ARRAY['read', 'write'], -- permissions: read, write, admin
  last_used_at TIMESTAMP WITH TIME ZONE,
  last_used_ip INET,
  expires_at TIMESTAMP WITH TIME ZONE,
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

-- Create download stats table for tracking package downloads
CREATE TABLE IF NOT EXISTS public.download_stats (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  agent_id UUID NOT NULL REFERENCES public.agents(id) ON DELETE CASCADE,
  version_id UUID REFERENCES public.agent_versions(id) ON DELETE SET NULL,
  package_id UUID REFERENCES public.agent_packages(id) ON DELETE SET NULL,
  user_id UUID REFERENCES auth.users(id) ON DELETE SET NULL, -- null for anonymous downloads
  ip_address INET,
  user_agent TEXT,
  referer TEXT,
  country_code CHAR(2),
  downloaded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  file_size BIGINT
);

-- Create user_follows table for following agents/authors
CREATE TABLE IF NOT EXISTS public.user_follows (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  follower_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  following_user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
  following_agent_id UUID REFERENCES public.agents(id) ON DELETE CASCADE,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  UNIQUE(follower_id, following_user_id),
  UNIQUE(follower_id, following_agent_id),
  CHECK (
    (following_user_id IS NOT NULL AND following_agent_id IS NULL) OR
    (following_user_id IS NULL AND following_agent_id IS NOT NULL)
  )
);

-- Create agent_ratings table for user ratings and reviews
CREATE TABLE IF NOT EXISTS public.agent_ratings (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  agent_id UUID NOT NULL REFERENCES public.agents(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
  rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
  review TEXT,
  helpful_count INTEGER DEFAULT 0,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  UNIQUE(agent_id, user_id)
);

-- Create rate limiting table for API calls
CREATE TABLE IF NOT EXISTS public.rate_limits (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  identifier TEXT NOT NULL, -- IP address or user ID
  endpoint TEXT NOT NULL,
  request_count INTEGER DEFAULT 1,
  window_start TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
  UNIQUE(identifier, endpoint, window_start)
);

-- Create webhook event log table for external integrations
CREATE TABLE IF NOT EXISTS public.webhook_events (
  id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  event_type TEXT NOT NULL, -- 'agent.published', 'agent.updated', 'agent.downloaded'
  agent_id UUID REFERENCES public.agents(id) ON DELETE SET NULL,
  version_id UUID REFERENCES public.agent_versions(id) ON DELETE SET NULL,
  user_id UUID REFERENCES auth.users(id) ON DELETE SET NULL,
  payload JSONB NOT NULL,
  processed BOOLEAN DEFAULT false,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now()
);

-- Add foreign key constraint for latest_version_id (do this after creating agent_versions)
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.table_constraints 
    WHERE constraint_name = 'fk_agents_latest_version'
  ) THEN
    ALTER TABLE public.agents 
      ADD CONSTRAINT fk_agents_latest_version 
      FOREIGN KEY (latest_version_id) REFERENCES public.agent_versions(id);
  END IF;
END
$$;

-- Enable RLS on all new tables
ALTER TABLE public.agent_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.agent_packages ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.api_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.download_stats ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.user_follows ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.agent_ratings ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.rate_limits ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.webhook_events ENABLE ROW LEVEL SECURITY;