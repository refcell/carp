import { useState, useEffect } from 'react';
import { supabase } from '@/integrations/supabase/client';
import { useAuth } from './useAuth';

export interface Agent {
  id: string;
  name: string;
  description: string;
  definition: any;
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
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');
  const { user } = useAuth();

  const fetchAgents = async (search?: string) => {
    setLoading(true);
    let query = supabase
      .from('agents')
      .select(`
        *,
        profiles (
          github_username,
          display_name,
          avatar_url
        )
      `)
      .eq('is_public', true)
      .order('created_at', { ascending: false });

    if (search) {
      query = query.or(`name.ilike.%${search}%,description.ilike.%${search}%`);
    }

    const { data, error } = await query;
    
    if (!error && data) {
      setAgents(data as unknown as Agent[]);
    }
    setLoading(false);
  };

  const fetchUserAgents = async () => {
    if (!user) return [];
    
    const { data, error } = await supabase
      .from('agents')
      .select('*')
      .eq('user_id', user.id)
      .order('created_at', { ascending: false });
    
    return error ? [] : (data as Agent[]);
  };

  const incrementViewCount = async (agentId: string) => {
    // We'll create this RPC function later
    const { data: agent } = await supabase
      .from('agents')
      .select('view_count')
      .eq('id', agentId)
      .single();
    
    if (agent) {
      await supabase
        .from('agents')
        .update({ view_count: agent.view_count + 1 })
        .eq('id', agentId);
    }
  };

  const createAgent = async (agent: Omit<Agent, 'id' | 'user_id' | 'view_count' | 'created_at' | 'updated_at'>) => {
    if (!user) throw new Error('Must be logged in to create agents');
    
    const { data, error } = await supabase
      .from('agents')
      .insert([{ ...agent, user_id: user.id }])
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

  useEffect(() => {
    fetchAgents(searchQuery);
  }, [searchQuery]);

  const trendingAgents = agents
    .sort((a, b) => b.view_count - a.view_count)
    .slice(0, 5);

  const latestAgents = agents.slice(0, 10);

  const topAgents = agents
    .sort((a, b) => b.view_count - a.view_count)
    .slice(0, 10);

  return {
    agents,
    loading,
    searchQuery,
    setSearchQuery,
    trendingAgents,
    latestAgents,
    topAgents,
    fetchAgents,
    fetchUserAgents,
    incrementViewCount,
    createAgent,
    updateAgent,
    deleteAgent
  };
}