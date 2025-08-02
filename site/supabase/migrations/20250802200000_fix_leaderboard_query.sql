-- Create a function to get user leaderboard data
CREATE OR REPLACE FUNCTION public.get_user_leaderboard()
RETURNS TABLE (
  user_id UUID,
  github_username TEXT,
  display_name TEXT,
  avatar_url TEXT,
  agent_count BIGINT
)
LANGUAGE sql
STABLE
AS $$
  SELECT 
    p.user_id,
    p.github_username,
    p.display_name,
    p.avatar_url,
    COUNT(a.id) as agent_count
  FROM public.profiles p
  INNER JOIN public.agents a ON a.user_id = p.user_id
  WHERE a.is_public = true
  GROUP BY p.user_id, p.github_username, p.display_name, p.avatar_url
  HAVING COUNT(a.id) > 0
  ORDER BY COUNT(a.id) DESC
  LIMIT 10;
$$;

-- Grant access to the function
GRANT EXECUTE ON FUNCTION public.get_user_leaderboard() TO anon;
GRANT EXECUTE ON FUNCTION public.get_user_leaderboard() TO authenticated;