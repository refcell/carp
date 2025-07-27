-- Comprehensive package management schema extension
-- This migration extends the existing agents and profiles tables with full package management capabilities

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

-- RLS policies for agent_versions
CREATE POLICY "Agent versions are viewable by everyone for public agents"
ON public.agent_versions 
FOR SELECT 
USING (
  EXISTS(
    SELECT 1 FROM public.agents 
    WHERE agents.id = agent_versions.agent_id 
    AND (agents.is_public = true OR agents.user_id = auth.uid())
  )
);

CREATE POLICY "Users can create versions for their own agents"
ON public.agent_versions 
FOR INSERT 
WITH CHECK (
  EXISTS(
    SELECT 1 FROM public.agents 
    WHERE agents.id = agent_versions.agent_id 
    AND agents.user_id = auth.uid()
  )
);

CREATE POLICY "Users can update versions of their own agents"
ON public.agent_versions 
FOR UPDATE 
USING (
  EXISTS(
    SELECT 1 FROM public.agents 
    WHERE agents.id = agent_versions.agent_id 
    AND agents.user_id = auth.uid()
  )
);

CREATE POLICY "Users can delete versions of their own agents"
ON public.agent_versions 
FOR DELETE 
USING (
  EXISTS(
    SELECT 1 FROM public.agents 
    WHERE agents.id = agent_versions.agent_id 
    AND agents.user_id = auth.uid()
  )
);

-- RLS policies for agent_packages
CREATE POLICY "Agent packages are viewable by everyone for public agents"
ON public.agent_packages 
FOR SELECT 
USING (
  EXISTS(
    SELECT 1 FROM public.agent_versions av
    JOIN public.agents a ON a.id = av.agent_id
    WHERE av.id = agent_packages.version_id 
    AND (a.is_public = true OR a.user_id = auth.uid())
  )
);

CREATE POLICY "Users can create packages for their own agent versions"
ON public.agent_packages 
FOR INSERT 
WITH CHECK (
  EXISTS(
    SELECT 1 FROM public.agent_versions av
    JOIN public.agents a ON a.id = av.agent_id
    WHERE av.id = agent_packages.version_id 
    AND a.user_id = auth.uid()
  )
);

CREATE POLICY "Users can update packages for their own agent versions"
ON public.agent_packages 
FOR UPDATE 
USING (
  EXISTS(
    SELECT 1 FROM public.agent_versions av
    JOIN public.agents a ON a.id = av.agent_id
    WHERE av.id = agent_packages.version_id 
    AND a.user_id = auth.uid()
  )
);

CREATE POLICY "Users can delete packages for their own agent versions"
ON public.agent_packages 
FOR DELETE 
USING (
  EXISTS(
    SELECT 1 FROM public.agent_versions av
    JOIN public.agents a ON a.id = av.agent_id
    WHERE av.id = agent_packages.version_id 
    AND a.user_id = auth.uid()
  )
);

-- RLS policies for api_tokens
CREATE POLICY "Users can view their own API tokens"
ON public.api_tokens 
FOR SELECT 
USING (auth.uid() = user_id);

CREATE POLICY "Users can create their own API tokens"
ON public.api_tokens 
FOR INSERT 
WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own API tokens"
ON public.api_tokens 
FOR UPDATE 
USING (auth.uid() = user_id);

CREATE POLICY "Users can delete their own API tokens"
ON public.api_tokens 
FOR DELETE 
USING (auth.uid() = user_id);

-- RLS policies for download_stats (read-only for users, write for system)
CREATE POLICY "Users can view download stats for their own agents"
ON public.download_stats 
FOR SELECT 
USING (
  EXISTS(
    SELECT 1 FROM public.agents 
    WHERE agents.id = download_stats.agent_id 
    AND agents.user_id = auth.uid()
  )
);

CREATE POLICY "System can insert download stats"
ON public.download_stats 
FOR INSERT 
WITH CHECK (true); -- Allow system to track downloads

-- RLS policies for user_follows
CREATE POLICY "Users can view their own follows"
ON public.user_follows 
FOR SELECT 
USING (auth.uid() = follower_id);

CREATE POLICY "Users can create their own follows"
ON public.user_follows 
FOR INSERT 
WITH CHECK (auth.uid() = follower_id);

CREATE POLICY "Users can delete their own follows"
ON public.user_follows 
FOR DELETE 
USING (auth.uid() = follower_id);

-- RLS policies for agent_ratings
CREATE POLICY "Agent ratings are viewable by everyone"
ON public.agent_ratings 
FOR SELECT 
USING (true);

CREATE POLICY "Users can create their own ratings"
ON public.agent_ratings 
FOR INSERT 
WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own ratings"
ON public.agent_ratings 
FOR UPDATE 
USING (auth.uid() = user_id);

CREATE POLICY "Users can delete their own ratings"
ON public.agent_ratings 
FOR DELETE 
USING (auth.uid() = user_id);

-- System-only policies for rate_limits and webhook_events
CREATE POLICY "System only access to rate limits"
ON public.rate_limits 
FOR ALL
USING (false); -- Deny all user access, only functions can access

CREATE POLICY "System only access to webhook events"
ON public.webhook_events 
FOR ALL
USING (false);

-- Create triggers for automatic timestamp updates on new tables
CREATE TRIGGER update_agent_versions_updated_at
  BEFORE UPDATE ON public.agent_versions
  FOR EACH ROW
  EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_api_tokens_updated_at
  BEFORE UPDATE ON public.api_tokens
  FOR EACH ROW
  EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_agent_ratings_updated_at
  BEFORE UPDATE ON public.agent_ratings
  FOR EACH ROW
  EXECUTE FUNCTION public.update_updated_at_column();

-- Create performance indexes
CREATE INDEX IF NOT EXISTS idx_agent_versions_agent_id ON public.agent_versions(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_versions_version ON public.agent_versions(agent_id, version);
CREATE INDEX IF NOT EXISTS idx_agent_versions_created_at ON public.agent_versions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_versions_download_count ON public.agent_versions(download_count DESC);
CREATE INDEX IF NOT EXISTS idx_agent_packages_version_id ON public.agent_packages(version_id);
CREATE INDEX IF NOT EXISTS idx_agents_current_version ON public.agents(current_version);
CREATE INDEX IF NOT EXISTS idx_agents_download_count ON public.agents(download_count DESC);
CREATE INDEX IF NOT EXISTS idx_agents_author_name ON public.agents(author_name);
CREATE INDEX IF NOT EXISTS idx_agents_keywords ON public.agents USING GIN(keywords);

CREATE INDEX IF NOT EXISTS idx_api_tokens_user_id ON public.api_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_api_tokens_token_hash ON public.api_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_api_tokens_token_prefix ON public.api_tokens(token_prefix);
CREATE INDEX IF NOT EXISTS idx_api_tokens_expires_at ON public.api_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_api_tokens_is_active ON public.api_tokens(is_active);

CREATE INDEX IF NOT EXISTS idx_download_stats_agent_id ON public.download_stats(agent_id);
CREATE INDEX IF NOT EXISTS idx_download_stats_version_id ON public.download_stats(version_id);
CREATE INDEX IF NOT EXISTS idx_download_stats_downloaded_at ON public.download_stats(downloaded_at DESC);
CREATE INDEX IF NOT EXISTS idx_download_stats_user_id ON public.download_stats(user_id);
CREATE INDEX IF NOT EXISTS idx_download_stats_ip_address ON public.download_stats(ip_address);

CREATE INDEX IF NOT EXISTS idx_user_follows_follower_id ON public.user_follows(follower_id);
CREATE INDEX IF NOT EXISTS idx_user_follows_following_user_id ON public.user_follows(following_user_id);
CREATE INDEX IF NOT EXISTS idx_user_follows_following_agent_id ON public.user_follows(following_agent_id);

CREATE INDEX IF NOT EXISTS idx_agent_ratings_agent_id ON public.agent_ratings(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_ratings_user_id ON public.agent_ratings(user_id);
CREATE INDEX IF NOT EXISTS idx_agent_ratings_rating ON public.agent_ratings(rating);
CREATE INDEX IF NOT EXISTS idx_agent_ratings_created_at ON public.agent_ratings(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_rate_limits_identifier_endpoint ON public.rate_limits(identifier, endpoint);
CREATE INDEX IF NOT EXISTS idx_rate_limits_window_start ON public.rate_limits(window_start);
CREATE INDEX IF NOT EXISTS idx_rate_limits_created_at ON public.rate_limits(created_at);

CREATE INDEX IF NOT EXISTS idx_webhook_events_type ON public.webhook_events(event_type);
CREATE INDEX IF NOT EXISTS idx_webhook_events_processed ON public.webhook_events(processed, created_at) WHERE NOT processed;
CREATE INDEX IF NOT EXISTS idx_webhook_events_agent_id ON public.webhook_events(agent_id);

-- Create an immutable function for search text generation
CREATE OR REPLACE FUNCTION public.agent_search_text(
  name TEXT,
  description TEXT,
  author_name TEXT,
  tags TEXT[],
  keywords TEXT[]
) RETURNS TEXT
LANGUAGE sql
IMMUTABLE
AS $$
  SELECT COALESCE(name, '') || ' ' || 
         COALESCE(description, '') || ' ' || 
         COALESCE(author_name, '') || ' ' ||
         COALESCE(array_to_string(tags, ' '), '') || ' ' ||
         COALESCE(array_to_string(keywords, ' '), '')
$$;

-- Update the existing search index to use the immutable function
-- First drop the old index name and create the new one
DROP INDEX IF EXISTS idx_agents_name_search;
CREATE INDEX IF NOT EXISTS idx_agents_search ON public.agents USING GIN(
  to_tsvector('english', public.agent_search_text(name, description, author_name, tags, keywords))
);