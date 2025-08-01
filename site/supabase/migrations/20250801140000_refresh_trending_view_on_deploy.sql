-- Refresh trending materialized view on deployment
-- This ensures the view is populated with current data when the app starts

-- First, ensure the view exists (should be created by earlier migration)
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_matviews 
    WHERE matviewname = 'trending_agents_mv' AND schemaname = 'public'
  ) THEN
    RAISE EXCEPTION 'trending_agents_mv materialized view does not exist. Run migration 20250801120000 first.';
  END IF;
END
$$;

-- Refresh the materialized view with current data
-- Use non-concurrent refresh to ensure it works even if there are no agents yet
REFRESH MATERIALIZED VIEW public.trending_agents_mv;

-- Verify the view has the expected structure
DO $$
DECLARE
  col_count integer;
BEGIN
  SELECT COUNT(*) INTO col_count 
  FROM information_schema.columns 
  WHERE table_schema = 'public' 
    AND table_name = 'trending_agents_mv'
    AND column_name IN (
      'id', 'name', 'current_version', 'description', 'author_name',
      'created_at', 'updated_at', 'download_count', 'tags', 'trending_score'
    );
  
  IF col_count < 10 THEN
    RAISE WARNING 'trending_agents_mv may be missing expected columns. Expected 10, found %', col_count;
  END IF;
END
$$;

-- Create a trigger to auto-refresh the materialized view when agents are updated
-- This ensures trending data stays reasonably current
CREATE OR REPLACE FUNCTION public.refresh_trending_on_agent_update()
RETURNS trigger
LANGUAGE plpgsql
SECURITY DEFINER
AS $$
BEGIN
  -- Only refresh if this is a significant change (download count, public status, etc.)
  IF (TG_OP = 'UPDATE' AND (
    OLD.download_count != NEW.download_count OR
    OLD.is_public != NEW.is_public OR
    OLD.updated_at < NEW.updated_at - interval '1 hour'
  )) OR TG_OP = 'INSERT' THEN
    -- Use a background job to refresh (non-blocking)
    PERFORM pg_notify('refresh_trending_view', '');
  END IF;
  
  RETURN COALESCE(NEW, OLD);
END
$$;

-- Create trigger on agents table
DROP TRIGGER IF EXISTS trigger_refresh_trending_on_agent_update ON public.agents;
CREATE TRIGGER trigger_refresh_trending_on_agent_update
  AFTER INSERT OR UPDATE ON public.agents
  FOR EACH ROW
  EXECUTE FUNCTION public.refresh_trending_on_agent_update();

-- Grant permissions for the trigger function
GRANT EXECUTE ON FUNCTION public.refresh_trending_on_agent_update() TO service_role;

-- Add a scheduled refresh job (if supported by hosting platform)
-- This is a fallback to ensure the view gets refreshed regularly
INSERT INTO cron.job (jobname, schedule, command)
VALUES (
  'refresh_trending_agents_mv',
  '0 */2 * * *',  -- Every 2 hours
  'SELECT public.refresh_trending_view_job();'
)
ON CONFLICT (jobname) DO UPDATE SET
  schedule = EXCLUDED.schedule,
  command = EXCLUDED.command,
  active = true;

-- Log successful refresh
DO $$
DECLARE
  view_count integer;
BEGIN
  SELECT COUNT(*) INTO view_count FROM public.trending_agents_mv;
  RAISE NOTICE 'trending_agents_mv refreshed successfully with % rows', view_count;
END
$$;