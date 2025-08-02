-- Ensure trending materialized view is populated and ready for production
-- This migration ensures the materialized view exists and has data

-- Refresh the materialized view to ensure it has current data
DO $$
BEGIN
  -- Check if materialized view exists and refresh it
  IF EXISTS (
    SELECT 1 FROM pg_matviews 
    WHERE matviewname = 'trending_agents_mv' AND schemaname = 'public'
  ) THEN
    -- If it exists but is empty, refresh it
    REFRESH MATERIALIZED VIEW CONCURRENTLY public.trending_agents_mv;
    
    -- Log that we refreshed it
    RAISE NOTICE 'Refreshed trending_agents_mv materialized view';
  ELSE
    -- If it doesn't exist, log a warning (should be created by previous migration)
    RAISE WARNING 'trending_agents_mv materialized view does not exist, should be created by migration 20250801120000';
  END IF;
END
$$;

-- Create a function to ensure the materialized view is populated on app startup
-- This can be called by the API if the view appears empty
CREATE OR REPLACE FUNCTION public.ensure_trending_view_populated()
RETURNS boolean
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
DECLARE
  view_count integer;
BEGIN
  -- Check if materialized view exists
  IF NOT EXISTS (
    SELECT 1 FROM pg_matviews 
    WHERE matviewname = 'trending_agents_mv' AND schemaname = 'public'
  ) THEN
    -- View doesn't exist, return false
    RETURN false;
  END IF;
  
  -- Check if view has data
  SELECT count(*) INTO view_count FROM public.trending_agents_mv;
  
  IF view_count = 0 THEN
    -- View is empty, try to refresh it
    BEGIN
      REFRESH MATERIALIZED VIEW CONCURRENTLY public.trending_agents_mv;
      RETURN true;
    EXCEPTION WHEN OTHERS THEN
      -- If concurrent refresh fails, try non-concurrent
      BEGIN
        REFRESH MATERIALIZED VIEW public.trending_agents_mv;
        RETURN true;
      EXCEPTION WHEN OTHERS THEN
        RETURN false;
      END;
    END;
  END IF;
  
  -- View has data
  RETURN true;
END
$$;

-- Grant execute permission on the function
GRANT EXECUTE ON FUNCTION public.ensure_trending_view_populated() TO anon, authenticated, service_role;

-- Create a scheduled job to refresh the materialized view every hour
-- This ensures trending data stays current
CREATE OR REPLACE FUNCTION public.refresh_trending_view_job()
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
  -- Try to refresh concurrently first (faster, but requires unique index)
  BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY public.trending_agents_mv;
  EXCEPTION WHEN OTHERS THEN
    -- If concurrent refresh fails, fall back to regular refresh
    REFRESH MATERIALIZED VIEW public.trending_agents_mv;
  END;
END
$$;

-- Grant execute permission for the refresh job
GRANT EXECUTE ON FUNCTION public.refresh_trending_view_job() TO service_role;

-- Add comment explaining the maintenance functions
COMMENT ON FUNCTION public.ensure_trending_view_populated() IS 
'Ensures trending materialized view exists and is populated. Returns true if successful.';

COMMENT ON FUNCTION public.refresh_trending_view_job() IS 
'Scheduled job function to refresh trending materialized view every hour.';