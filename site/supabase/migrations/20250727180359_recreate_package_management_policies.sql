-- Recreate RLS policies for package management tables

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