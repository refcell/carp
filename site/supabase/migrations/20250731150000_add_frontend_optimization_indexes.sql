-- Frontend optimization indexes for efficient agent queries
-- These indexes optimize the specific query patterns used by the frontend React app

-- 1. Composite index for trending/top agents by view_count with created_at as tiebreaker
-- This supports: ORDER BY view_count DESC, created_at DESC for public agents
CREATE INDEX IF NOT EXISTS idx_agents_trending_optimized 
ON public.agents(is_public, view_count DESC, created_at DESC) 
WHERE is_public = true;

-- 2. Index for latest agents by creation date
-- This supports: ORDER BY created_at DESC for public agents  
CREATE INDEX IF NOT EXISTS idx_agents_latest_optimized
ON public.agents(is_public, created_at DESC) 
WHERE is_public = true;

-- 3. Composite index for filtering public agents by view_count
-- This supports queries that need to filter public agents and sort by views
CREATE INDEX IF NOT EXISTS idx_agents_public_views_optimized
ON public.agents(is_public, view_count DESC)
WHERE is_public = true;

-- 4. Index for efficient user agent counting and lookups
-- This supports counting agents per user and filtering by public status
CREATE INDEX IF NOT EXISTS idx_agents_user_public_optimized
ON public.agents(user_id, is_public);

-- 5. Index for search optimization with public filter
-- This supports text search queries filtered by public status
CREATE INDEX IF NOT EXISTS idx_agents_public_search_optimized  
ON public.agents(is_public, name, description)
WHERE is_public = true;

-- 6. Additional composite index for profile joins optimization
-- This helps with the common pattern of joining agents with profiles for public agents
CREATE INDEX IF NOT EXISTS idx_agents_public_user_created
ON public.agents(is_public, user_id, created_at DESC)
WHERE is_public = true;

-- Analyze tables to update statistics for query planner
ANALYZE public.agents;
ANALYZE public.profiles;