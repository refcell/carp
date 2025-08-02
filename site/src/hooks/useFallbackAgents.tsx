import { useQuery } from '@tanstack/react-query';
import { supabase } from '@/integrations/supabase/client';
import { Agent } from './useOptimizedAgents';

/**
 * Fallback hook that uses direct Supabase queries if optimized endpoints fail
 * This ensures the app still works even if the API endpoints are down
 */
export function useFallbackLatestAgents(limit: number = 10) {
  return useQuery({
    queryKey: ['agents', 'latest', 'fallback', limit],
    queryFn: async (): Promise<Agent[]> => {
      console.log('[useFallbackLatestAgents] Using Supabase fallback');
      
      const { data, error } = await supabase
        .from('agents')
        .select(`
          *,
          profiles!inner(
            github_username,
            display_name,
            avatar_url
          )
        `)
        .eq('is_public', true)
        .order('created_at', { ascending: false })
        .limit(limit);

      if (error) {
        console.error('[useFallbackLatestAgents] Supabase error:', error);
        return [];
      }

      return data || [];
    },
    enabled: false, // Only enable when needed
    staleTime: 1 * 60 * 1000,
    gcTime: 5 * 60 * 1000,
  });
}

export function useFallbackTrendingAgents(limit: number = 10) {
  return useQuery({
    queryKey: ['agents', 'trending', 'fallback', limit],
    queryFn: async (): Promise<Agent[]> => {
      console.log('[useFallbackTrendingAgents] Using Supabase fallback');
      
      const { data, error } = await supabase
        .from('agents')
        .select(`
          *,
          profiles!inner(
            github_username,
            display_name,
            avatar_url
          )
        `)
        .eq('is_public', true)
        .order('view_count', { ascending: false })
        .limit(limit);

      if (error) {
        console.error('[useFallbackTrendingAgents] Supabase error:', error);
        return [];
      }

      return data || [];
    },
    enabled: false, // Only enable when needed
    staleTime: 5 * 60 * 1000,
    gcTime: 10 * 60 * 1000,
  });
}