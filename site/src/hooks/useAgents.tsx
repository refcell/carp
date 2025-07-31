import { useState, useEffect, useCallback } from 'react';
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
  const [allAgents, setAllAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchLoading, setSearchLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const { user } = useAuth();

  const fetchAgents = useCallback(async (search?: string, limit?: number, offset?: number) => {
    if (search && search.trim()) {
      setSearchLoading(true);
    } else {
      setLoading(true);
    }
    try {
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

      const { data, error } = await query;
      
      if (!error && data) {
        // Fetch profile data for each agent if needed
        const agentsWithProfiles = await Promise.all(
          data.map(async (agent: Agent) => {
            try {
              const { data: profile } = await supabase
                .from('profiles')
                .select('github_username, display_name, avatar_url')
                .eq('user_id', agent.user_id)
                .single();
              
              return {
                ...agent,
                profiles: profile || null
              };
            } catch {
              return {
                ...agent,
                profiles: null
              };
            }
          })
        );
        
        const typedAgents = agentsWithProfiles as Agent[];
        if (search && search.trim()) {
          setSearchResults(typedAgents);
        } else {
          setAllAgents(typedAgents);
        }
      } else if (error) {
        console.error('Error fetching agents:', error);
        if (search && search.trim()) {
          setSearchResults([]);
        } else {
          setAllAgents([]);
        }
      }
    } catch (error) {
      console.error('Error fetching agents:', error);
      if (search && search.trim()) {
        setSearchResults([]);
      } else {
        setAllAgents([]);
      }
    } finally {
      if (search && search.trim()) {
        setSearchLoading(false);
      } else {
        setLoading(false);
      }
    }
  }, []);

  const fetchAgentsPaginated = useCallback(async (limit: number = 20, offset: number = 0, search?: string) => {
    try {
      let query = supabase
        .from('agents')
        .select('*')
        .eq('is_public', true)
        .order('created_at', { ascending: false });

      if (search && search.trim()) {
        const searchTerm = search.trim();
        query = query.or(`name.ilike.%${searchTerm}%,description.ilike.%${searchTerm}%,tags.cs.{${searchTerm}}`);
      }

      // Fetch one extra item to check if there are more
      query = query.range(offset, offset + limit);

      const { data, error } = await query;
      
      if (error) {
        console.error('Error fetching paginated agents:', error);
        return { agents: [], hasMore: false };
      }

      if (!data) {
        return { agents: [], hasMore: false };
      }

      // Check if we have more items by comparing with the expected limit + 1
      const hasMore = data.length === limit + 1;
      const actualData = hasMore ? data.slice(0, limit) : data;

      // Fetch profile data for each agent
      const agentsWithProfiles = await Promise.all(
        actualData.map(async (agent: Agent) => {
          try {
            const { data: profile } = await supabase
              .from('profiles')
              .select('github_username, display_name, avatar_url')
              .eq('user_id', agent.user_id)
              .single();
            
            return {
              ...agent,
              profiles: profile || null
            };
          } catch {
            return {
              ...agent,
              profiles: null
            };
          }
        })
      );
      
      return { agents: agentsWithProfiles as Agent[], hasMore };
    } catch (error) {
      console.error('Error fetching paginated agents:', error);
      return { agents: [], hasMore: false };
    }
  }, []);

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
    console.log(`🔍 Incrementing view count for agent: ${agentId}`);
    
    // First update local state optimistically
    setAgents(prevAgents => 
      prevAgents.map(agent => 
        agent.id === agentId 
          ? { ...agent, view_count: agent.view_count + 1 }
          : agent
      )
    );

    // Then update the database using a more reliable approach
    try {
      console.log('📡 Updating database...');
      
      // Use PostgreSQL's atomic increment instead of read-then-write
      const { data, error } = await supabase.rpc('increment_view_count', {
        agent_id: agentId
      });

      if (error) {
        console.error('❌ Database RPC error:', error);
        throw error;
      }

      console.log('✅ Database updated successfully');
      return data;
    } catch (error) {
      console.error('❌ Error incrementing view count:', error);
      
      // Fallback to manual update if RPC doesn't exist
      try {
        console.log('🔄 Trying fallback database update...');
        
        const { data: currentAgent, error: fetchError } = await supabase
          .from('agents')
          .select('view_count')
          .eq('id', agentId)
          .single();
        
        if (fetchError) {
          console.error('❌ Error fetching current agent:', fetchError);
          throw fetchError;
        }

        console.log(`📊 Current view count in DB: ${currentAgent.view_count}`);
        
        const { data: updatedAgent, error: updateError } = await supabase
          .from('agents')
          .update({ view_count: currentAgent.view_count + 1 })
          .eq('id', agentId)
          .select('view_count')
          .single();
        
        if (updateError) {
          console.error('❌ Error updating view count:', updateError);
          throw updateError;
        }

        console.log(`✅ Fallback update successful, new count: ${updatedAgent.view_count}`);
        return updatedAgent;
        
      } catch (fallbackError) {
        console.error('❌ Fallback update also failed:', fallbackError);
        
        // Revert optimistic update on error
        setAgents(prevAgents => 
          prevAgents.map(agent => 
            agent.id === agentId 
              ? { ...agent, view_count: Math.max(0, agent.view_count - 1) }
              : agent
          )
        );
        throw fallbackError;
      }
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

  // Debounced search effect
  useEffect(() => {
    if (!searchQuery || !searchQuery.trim()) {
      setSearchResults([]);
      setSearchLoading(false);
      return;
    }

    const timeoutId = setTimeout(() => {
      fetchAgents(searchQuery);
    }, 200); // Reduced debounce for more responsiveness

    return () => clearTimeout(timeoutId);
  }, [searchQuery, fetchAgents]);

  // Initial load and real-time subscription
  useEffect(() => {
    // Initial fetch
    fetchAgents();

    // Set up real-time subscription for new agents
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
          // Use a slight delay to ensure database consistency
          setTimeout(() => {
            fetchAgents(); // Only refresh all agents, not search
          }, 100);
        }
      )
      .subscribe();

    return () => {
      supabase.removeChannel(channel);
    };
  }, [fetchAgents]); // Remove searchQuery dependency to avoid conflicts

  // Dashboard sections always use allAgents (never affected by search)
  const trendingAgents = allAgents
    .sort((a, b) => b.view_count - a.view_count)
    .slice(0, 5);

  const latestAgents = allAgents
    .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
    .slice(0, 10);

  const topAgents = allAgents
    .sort((a, b) => b.view_count - a.view_count)
    .slice(0, 10);

  // Search results are separate
  const agents = searchQuery ? searchResults : allAgents;

  const refreshAgents = useCallback(() => {
    fetchAgents(searchQuery);
  }, [fetchAgents, searchQuery]);

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