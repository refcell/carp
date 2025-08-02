-- Add advanced trending score function for better performance
-- This function calculates a trending score based on downloads and recency

-- Create function to calculate trending score
CREATE OR REPLACE FUNCTION public.calculate_trending_score(
  download_count BIGINT,
  created_at TIMESTAMP WITH TIME ZONE,
  updated_at TIMESTAMP WITH TIME ZONE
) RETURNS NUMERIC
LANGUAGE sql
IMMUTABLE
AS $$
  SELECT CASE 
    WHEN download_count = 0 THEN 0
    ELSE 
      -- Base score from downloads (log scale to prevent outliers from dominating)
      LN(download_count + 1) * 10 +
      -- Recency bonus: newer agents get higher scores
      (
        GREATEST(0, 7 - EXTRACT(DAYS FROM (NOW() - created_at))) * 2
      ) +
      -- Activity bonus: recently updated agents get bonus
      (
        GREATEST(0, 3 - EXTRACT(DAYS FROM (NOW() - updated_at))) * 1
      )
  END
$$;

-- Create materialized view for trending agents (refreshed periodically)
CREATE MATERIALIZED VIEW IF NOT EXISTS public.trending_agents_mv AS
SELECT 
  a.id,
  a.name,
  a.current_version,
  a.description,
  a.author_name,
  a.created_at,
  a.updated_at,
  a.download_count,
  a.tags,
  public.calculate_trending_score(a.download_count, a.created_at, a.updated_at) as trending_score
FROM public.agents a
WHERE a.is_public = true
  AND a.download_count > 0
ORDER BY trending_score DESC
LIMIT 100; -- Top 100 trending agents

-- Create index on the materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_trending_agents_mv_id 
ON public.trending_agents_mv(id);

CREATE INDEX IF NOT EXISTS idx_trending_agents_mv_score 
ON public.trending_agents_mv(trending_score DESC);

-- Function to refresh trending agents materialized view
CREATE OR REPLACE FUNCTION public.refresh_trending_agents()
RETURNS void
LANGUAGE sql
SECURITY DEFINER
AS $$
  REFRESH MATERIALIZED VIEW CONCURRENTLY public.trending_agents_mv;
$$;

-- Grant necessary permissions
GRANT SELECT ON public.trending_agents_mv TO anon, authenticated;
GRANT EXECUTE ON FUNCTION public.refresh_trending_agents() TO service_role;

-- Add comment explaining the trending algorithm
COMMENT ON FUNCTION public.calculate_trending_score IS 
'Calculates trending score using logarithmic download count, recency bonus (7 days), and activity bonus (3 days)';

COMMENT ON MATERIALIZED VIEW public.trending_agents_mv IS 
'Cached trending agents ranked by calculated trending score. Refreshed periodically for performance.';