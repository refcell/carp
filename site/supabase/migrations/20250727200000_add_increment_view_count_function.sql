-- Create atomic increment function for view counts
CREATE OR REPLACE FUNCTION increment_view_count(agent_id UUID)
RETURNS TABLE(new_view_count INTEGER) AS $$
BEGIN
  -- Atomically increment and return the new count
  UPDATE public.agents 
  SET view_count = view_count + 1 
  WHERE id = agent_id;
  
  -- Return the new view count
  RETURN QUERY 
  SELECT agents.view_count 
  FROM public.agents 
  WHERE agents.id = agent_id;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Grant execute permission to authenticated users
GRANT EXECUTE ON FUNCTION increment_view_count(UUID) TO authenticated;
GRANT EXECUTE ON FUNCTION increment_view_count(UUID) TO anon;