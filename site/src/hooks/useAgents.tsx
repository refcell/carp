import { useState, useEffect, useCallback, useMemo } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { supabase } from '@/integrations/supabase/client';
import { useAuth } from './useAuth';

export interface Agent {
  id: string;
  name: string;
  description: string;
  definition: Record<string, unknown>;
  tags: string[] | null;
  view_count: number;
  created_at: string;
  updated_at: string;
  user_id: string;
  is_public?: boolean;
  profiles?: {
    github_username: string | null;
    display_name: string | null;
    avatar_url: string | null;
  } | null;
}

export function useAgents() {
  const [searchResults, setSearchResults] = useState<Agent[]>([]);
  const [searchLoading, setSearchLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const { user } = useAuth();
  const queryClient = useQueryClient();

  // Optimized agents fetching with JOIN to eliminate N+1 queries
  const fetchAgentsWithProfiles = useCallback(async (search?: string, limit?: number, offset?: number) => {
    let query = supabase
      .from('agents')
      .select('*')
      .eq('is_public', true)
      .order('created_at', { ascending: false });

    if (search && search.trim()) {
      const searchTerm = search.trim();
      query = query.or(`name.ilike.%${searchTerm}%,description.ilike.%${searchTerm}%,tags.cs.{${searchTerm}}`);
    }

    if (limit !== undefined) {
      query = query.limit(limit);
    }

    if (offset !== undefined) {
      query = query.range(offset, offset + (limit || 20) - 1);
    }

    const { data: agents, error } = await query;
    
    if (error) {
      console.error('Error fetching agents:', error);
      throw error;
    }

    // If we have agents, fetch their profiles
    if (agents && agents.length > 0) {
      const userIds = [...new Set(agents.map(a => a.user_id))].filter(Boolean);
      
      if (userIds.length > 0) {
        const { data: profiles, error: profileError } = await supabase
          .from('profiles')
          .select('user_id, github_username, display_name, avatar_url')
          .in('user_id', userIds);
        
        if (!profileError && profiles) {
          // Create a map for quick lookup
          const profileMap = new Map(profiles.map(p => [p.user_id, p]));
          
          // Attach profiles to agents
          return agents.map(agent => ({
            ...agent,
            profiles: profileMap.get(agent.user_id) || null
          })) as Agent[];
        }
      }
    }

    return agents as Agent[];
  }, []);

  const fetchAgents = useCallback(async (search?: string, limit?: number, offset?: number) => {
    if (search && search.trim()) {
      setSearchLoading(true);
      try {
        const agents = await fetchAgentsWithProfiles(search, limit, offset);
        setSearchResults(agents);
      } catch (error) {
        console.error('Search error:', error);
        setSearchResults([]);
      } finally {
        setSearchLoading(false);
      }
    }
  }, [fetchAgentsWithProfiles]);

  const fetchAgentsPaginated = useCallback(async (limit: number = 20, offset: number = 0, search?: string) => {
    try {
      // Fetch one extra item to check if there are more
      const agents = await fetchAgentsWithProfiles(search, limit + 1, offset);
      
      // Check if we have more items by comparing with the expected limit + 1
      const hasMore = agents.length === limit + 1;
      const actualData = hasMore ? agents.slice(0, limit) : agents;
      
      return { agents: actualData, hasMore };
    } catch (error) {
      console.error('Error fetching paginated agents:', error);
      return { agents: [], hasMore: false };
    }
  }, [fetchAgentsWithProfiles]);

  const fetchUserAgents = async () => {
    if (!user) return [];
    
    const { data, error } = await supabase
      .from('agents')
      .select('*')
      .eq('user_id', user.id)
      .order('created_at', { ascending: false });
    
    if (error) {
      console.error('Error fetching user agents:', error);
      console.error('Error details:', {
        message: error.message,
        details: error.details,
        hint: error.hint,
        code: error.code
      });
      return [];
    }
    
    return data as Agent[];
  };

  const incrementViewCount = async (agentId: string) => {
    console.log(`🔍 [useAgents] Incrementing view count for agent: ${agentId}`);
    
    // Validate UUID format
    const uuidRegex = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;
    if (!agentId || !uuidRegex.test(agentId)) {
      console.error('❌ [useAgents] Invalid UUID format:', agentId);
      return;
    }
    
    // Store original data for rollback
    const originalData = queryClient.getQueryData(['agents', 'all']) as Agent[];
    
    // Optimistically update React Query cache
    queryClient.setQueryData(['agents', 'all'], (oldData: Agent[] = []) => {
      const updated = oldData.map(agent => 
        agent.id === agentId 
          ? { ...agent, view_count: agent.view_count + 1 }
          : agent
      );
      console.log(`📊 [useAgents] Optimistically updated cache: ${updated.find(a => a.id === agentId)?.view_count} views`);
      return updated;
    });

    // Update the database
    try {
      console.log('📡 [useAgents] Calling database RPC with UUID:', agentId);
      
      // Use PostgreSQL's atomic increment function
      const { data, error } = await supabase.rpc('increment_view_count', {
        agent_id: agentId
      });

      if (error) {
        console.error('❌ [useAgents] Database RPC error:', error);
        console.error('❌ [useAgents] Error details:', {
          message: error.message,
          details: error.details,
          hint: error.hint,
          code: error.code
        });
        throw error;
      }

      const newViewCount = data?.[0]?.new_view_count;
      console.log(`✅ [useAgents] Database updated successfully! New count: ${newViewCount}`);
      
      // Invalidate queries to ensure consistency
      queryClient.invalidateQueries({ queryKey: ['agents'] });
      
      return data;
    } catch (error) {
      console.error('❌ [useAgents] Error incrementing view count:', error);
      
      // Revert optimistic update on error
      if (originalData) {
        console.log('🔄 [useAgents] Reverting optimistic update...');
        queryClient.setQueryData(['agents', 'all'], originalData);
      }
      
      throw error;
    }
  };

  const createAgent = async (agent: Omit<Agent, 'id' | 'user_id' | 'view_count' | 'created_at' | 'updated_at'>) => {
    if (!user) throw new Error('Must be logged in to create agents');
    
    const { data, error } = await supabase
      .from('agents')
      .insert([{ 
        ...agent, 
        user_id: user.id,
        is_public: true, // Ensure agents are public by default
        view_count: 0 // Initialize view count
      }])
      .select()
      .single();
    
    if (error) throw error;
    return data;
  };

  const updateAgent = async (id: string, updates: Partial<Agent>) => {
    if (!user) throw new Error('Must be logged in to update agents');
    
    const { data, error } = await supabase
      .from('agents')
      .update(updates)
      .eq('id', id)
      .eq('user_id', user.id)
      .select()
      .single();
    
    if (error) throw error;
    return data;
  };

  const deleteAgent = async (id: string) => {
    if (!user) throw new Error('Must be logged in to delete agents');
    
    const { error } = await supabase
      .from('agents')
      .delete()
      .eq('id', id)
      .eq('user_id', user.id);
    
    if (error) throw error;
  };

  // Use React Query for caching and optimized data fetching
  const {
    data: allAgents = [],
    isLoading: loading,
    error: agentsError
  } = useQuery({
    queryKey: ['agents', 'all'],
    queryFn: () => fetchAgentsWithProfiles(),
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 10 * 60 * 1000, // 10 minutes
    refetchOnWindowFocus: false
  });

  // Debounced search effect with cleanup
  useEffect(() => {
    if (!searchQuery || !searchQuery.trim()) {
      setSearchResults([]);
      setSearchLoading(false);
      return;
    }

    const timeoutId = setTimeout(() => {
      fetchAgents(searchQuery);
    }, 300); // Slightly increased debounce to reduce API calls

    return () => clearTimeout(timeoutId);
  }, [searchQuery, fetchAgents]);

  // Optimized real-time subscription - only invalidate cache instead of refetching
  useEffect(() => {
    const channel = supabase
      .channel('agents_changes')
      .on(
        'postgres_changes',
        {
          event: '*',
          schema: 'public',
          table: 'agents',
          filter: 'is_public=eq.true'
        },
        () => {
          // Invalidate cache to trigger background refetch
          queryClient.invalidateQueries({ queryKey: ['agents', 'all'] });
        }
      )
      .subscribe();

    return () => {
      supabase.removeChannel(channel);
    };
  }, [queryClient]);

  // Memoized dashboard sections to prevent unnecessary re-computations
  const trendingAgents = useMemo(() => 
    allAgents
      .filter(agent => agent.view_count > 0) // Only show agents with views
      .sort((a, b) => b.view_count - a.view_count)
      .slice(0, 5),
    [allAgents]
  );

  const latestAgents = useMemo(() => 
    allAgents
      .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
      .slice(0, 10),
    [allAgents]
  );

  const topAgents = useMemo(() => 
    allAgents
      .filter(agent => agent.view_count > 0) // Only show agents with views
      .sort((a, b) => b.view_count - a.view_count)
      .slice(0, 10),
    [allAgents]
  );

  // Search results are separate
  const agents = searchQuery ? searchResults : allAgents;

  const refreshAgents = useCallback(() => {
    queryClient.invalidateQueries({ queryKey: ['agents', 'all'] });
    if (searchQuery) {
      fetchAgents(searchQuery);
    }
  }, [queryClient, searchQuery, fetchAgents]);

  return {
    agents,
    loading,
    searchLoading,
    searchQuery,
    setSearchQuery,
    trendingAgents,
    latestAgents,
    topAgents,
    fetchAgents,
    fetchAgentsPaginated,
    fetchUserAgents,
    incrementViewCount,
    createAgent,
    updateAgent,
    deleteAgent,
    refreshAgents
  };
}