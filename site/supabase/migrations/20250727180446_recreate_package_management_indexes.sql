-- Recreate performance indexes for package management tables

-- Create performance indexes for agent_versions
CREATE INDEX IF NOT EXISTS idx_agent_versions_agent_id ON public.agent_versions(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_versions_version ON public.agent_versions(agent_id, version);
CREATE INDEX IF NOT EXISTS idx_agent_versions_created_at ON public.agent_versions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_versions_download_count ON public.agent_versions(download_count DESC);

-- Create performance indexes for agent_packages
CREATE INDEX IF NOT EXISTS idx_agent_packages_version_id ON public.agent_packages(version_id);

-- Create performance indexes for extended agents table
CREATE INDEX IF NOT EXISTS idx_agents_current_version ON public.agents(current_version);
CREATE INDEX IF NOT EXISTS idx_agents_download_count ON public.agents(download_count DESC);
CREATE INDEX IF NOT EXISTS idx_agents_author_name ON public.agents(author_name);
CREATE INDEX IF NOT EXISTS idx_agents_keywords ON public.agents USING GIN(keywords);

-- Create performance indexes for api_tokens
CREATE INDEX IF NOT EXISTS idx_api_tokens_user_id ON public.api_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_api_tokens_token_hash ON public.api_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_api_tokens_token_prefix ON public.api_tokens(token_prefix);
CREATE INDEX IF NOT EXISTS idx_api_tokens_expires_at ON public.api_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_api_tokens_is_active ON public.api_tokens(is_active);

-- Create performance indexes for download_stats
CREATE INDEX IF NOT EXISTS idx_download_stats_agent_id ON public.download_stats(agent_id);
CREATE INDEX IF NOT EXISTS idx_download_stats_version_id ON public.download_stats(version_id);
CREATE INDEX IF NOT EXISTS idx_download_stats_downloaded_at ON public.download_stats(downloaded_at DESC);
CREATE INDEX IF NOT EXISTS idx_download_stats_user_id ON public.download_stats(user_id);
CREATE INDEX IF NOT EXISTS idx_download_stats_ip_address ON public.download_stats(ip_address);

-- Create performance indexes for user_follows
CREATE INDEX IF NOT EXISTS idx_user_follows_follower_id ON public.user_follows(follower_id);
CREATE INDEX IF NOT EXISTS idx_user_follows_following_user_id ON public.user_follows(following_user_id);
CREATE INDEX IF NOT EXISTS idx_user_follows_following_agent_id ON public.user_follows(following_agent_id);

-- Create performance indexes for agent_ratings
CREATE INDEX IF NOT EXISTS idx_agent_ratings_agent_id ON public.agent_ratings(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_ratings_user_id ON public.agent_ratings(user_id);
CREATE INDEX IF NOT EXISTS idx_agent_ratings_rating ON public.agent_ratings(rating);
CREATE INDEX IF NOT EXISTS idx_agent_ratings_created_at ON public.agent_ratings(created_at DESC);

-- Create performance indexes for rate_limits
CREATE INDEX IF NOT EXISTS idx_rate_limits_identifier_endpoint ON public.rate_limits(identifier, endpoint);
CREATE INDEX IF NOT EXISTS idx_rate_limits_window_start ON public.rate_limits(window_start);
CREATE INDEX IF NOT EXISTS idx_rate_limits_created_at ON public.rate_limits(created_at);

-- Create performance indexes for webhook_events
CREATE INDEX IF NOT EXISTS idx_webhook_events_type ON public.webhook_events(event_type);
CREATE INDEX IF NOT EXISTS idx_webhook_events_processed ON public.webhook_events(processed, created_at) WHERE NOT processed;
CREATE INDEX IF NOT EXISTS idx_webhook_events_agent_id ON public.webhook_events(agent_id);

-- Create additional composite indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_agents_public_downloads 
ON public.agents(is_public, download_count DESC) 
WHERE is_public = true;

CREATE INDEX IF NOT EXISTS idx_agents_public_created 
ON public.agents(is_public, created_at DESC) 
WHERE is_public = true;

CREATE INDEX IF NOT EXISTS idx_agents_public_updated 
ON public.agents(is_public, updated_at DESC) 
WHERE is_public = true;

CREATE INDEX IF NOT EXISTS idx_agents_user_name 
ON public.agents(user_id, name);

CREATE INDEX IF NOT EXISTS idx_agent_versions_agent_created 
ON public.agent_versions(agent_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_agent_versions_not_yanked 
ON public.agent_versions(agent_id, created_at DESC) 
WHERE yanked = false;

CREATE INDEX IF NOT EXISTS idx_download_stats_agent_date 
ON public.download_stats(agent_id, downloaded_at DESC);

-- Create partial indexes for active tokens
CREATE INDEX IF NOT EXISTS idx_api_tokens_active 
ON public.api_tokens(user_id, token_hash) 
WHERE is_active = true;