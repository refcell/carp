import { useQuery, useQueryClient } from '@tanstack/react-query';
import { getApiBaseUrl, API_ENDPOINTS } from '@/lib/api-config';
import { supabase } from '@/integrations/supabase/client';

// Types for the optimized API response format
export interface OptimizedAgent {
  name: string;
  current_version: string;
  description: string;
  author_name: string | null;
  created_at: string;
  updated_at: string;
  download_count: number;
  view_count: number;
  tags: string[] | null;
  definition: Record<string, unknown> | null;
  user_id: string;
  profiles?: {
    user_id: string;
    github_username: string | null;
    display_name: string | null;
    avatar_url: string | null;
  } | null;
}

export interface OptimizedAgentsResponse {
  agents: OptimizedAgent[];
  cached_at: string;
}

// Convert OptimizedAgent to the existing Agent interface for compatibility
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

// Convert optimized agent to legacy agent format for compatibility
const convertOptimizedAgent = (optimizedAgent: OptimizedAgent): Agent => {
  console.log('[convertOptimizedAgent] Converting:', optimizedAgent);
  return {
  id: `${optimizedAgent.name}-${optimizedAgent.current_version}`, // Generate ID from name+version
  name: optimizedAgent.name,
  description: optimizedAgent.description,
  definition: optimizedAgent.definition || { version: optimizedAgent.current_version }, // Use actual definition or fallback
  tags: optimizedAgent.tags,
  view_count: optimizedAgent.view_count, // Use actual view_count from API
  created_at: optimizedAgent.created_at,
  updated_at: optimizedAgent.updated_at,
  user_id: optimizedAgent.user_id || '', // Use actual user_id
  is_public: true, // Assume public since it's in the public API
  profiles: optimizedAgent.profiles || (optimizedAgent.author_name ? {
    github_username: null,
    display_name: optimizedAgent.author_name,
    avatar_url: null,
  } : null),
  };
};

/**
 * Hook to fetch latest agents using the optimized API endpoint
 * Uses React Query for caching and error handling
 */
export function useLatestAgents(limit: number = 10) {
  return useQuery({
    queryKey: ['agents', 'latest', limit],
    queryFn: async (): Promise<Agent[]> => {
      const baseUrl = getApiBaseUrl();
      const url = `${baseUrl}${API_ENDPOINTS.LATEST_AGENTS}?limit=${Math.min(limit, 50)}`;
      
      console.log('[useLatestAgents] Fetching from:', url);
      
      try {
        const response = await fetch(url, {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
          },
        });

        if (!response.ok) {
          console.error('[useLatestAgents] API error:', response.status, response.statusText);
          const errorText = await response.text();
          console.error('[useLatestAgents] Error details:', errorText);
          // Return empty array on error instead of throwing
          return [];
        }

        const text = await response.text();
        console.log('[useLatestAgents] Raw response:', text);
        
        let data: OptimizedAgentsResponse;
        try {
          data = JSON.parse(text);
          console.log('[useLatestAgents] Parsed data:', data);
        } catch (e) {
          console.error('[useLatestAgents] Failed to parse JSON:', e);
          console.error('[useLatestAgents] Raw text was:', text);
          return [];
        }
        
        // Convert optimized agents to legacy format for compatibility
        if (!data || !data.agents) {
          console.warn('[useLatestAgents] No agents array in response:', data);
          return [];
        }
        
        try {
          const converted = data.agents.map(agent => {
            try {
              return convertOptimizedAgent(agent);
            } catch (e) {
              console.error('[useLatestAgents] Failed to convert agent:', agent, e);
              return null;
            }
          }).filter(Boolean) as Agent[];
          console.log('[useLatestAgents] Successfully converted agents:', converted.length);
          return converted;
        } catch (e) {
          console.error('[useLatestAgents] Failed to convert agents:', e);
          return [];
        }
      } catch (error) {
        console.error('[useLatestAgents] Fetch error:', error);
        // Return empty array on error
        return [];
      }
    },
    staleTime: 1 * 60 * 1000, // 1 minute (matches API cache)
    gcTime: 5 * 60 * 1000, // 5 minutes
    refetchOnWindowFocus: false,
  });
}

/**
 * Hook to fetch trending agents using the optimized API endpoint  
 * Uses React Query for caching and error handling
 */
export function useTrendingAgents(limit: number = 10) {
  return useQuery({
    queryKey: ['agents', 'trending', limit],
    queryFn: async (): Promise<Agent[]> => {
      const baseUrl = getApiBaseUrl();
      const url = `${baseUrl}${API_ENDPOINTS.TRENDING_AGENTS}?limit=${Math.min(limit, 50)}`;
      
      console.log('[useTrendingAgents] Fetching from:', url);
      
      try {
        const response = await fetch(url, {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
          },
        });

        if (!response.ok) {
          console.error('[useTrendingAgents] API error:', response.status, response.statusText);
          const errorText = await response.text();
          console.error('[useTrendingAgents] Error details:', errorText);
          // Return empty array on error instead of throwing
          return [];
        }

        const text = await response.text();
        console.log('[useTrendingAgents] Raw response:', text);
        
        let data: OptimizedAgentsResponse;
        try {
          data = JSON.parse(text);
          console.log('[useTrendingAgents] Parsed data:', data);
        } catch (e) {
          console.error('[useTrendingAgents] Failed to parse JSON:', e);
          console.error('[useTrendingAgents] Raw text was:', text);
          return [];
        }
        
        // Convert optimized agents to legacy format for compatibility
        if (!data || !data.agents) {
          console.warn('[useTrendingAgents] No agents array in response:', data);
          return [];
        }
        
        try {
          const converted = data.agents.map(agent => {
            try {
              return convertOptimizedAgent(agent);
            } catch (e) {
              console.error('[useTrendingAgents] Failed to convert agent:', agent, e);
              return null;
            }
          }).filter(Boolean) as Agent[];
          console.log('[useTrendingAgents] Successfully converted agents:', converted.length);
          return converted;
        } catch (e) {
          console.error('[useTrendingAgents] Failed to convert agents:', e);
          return [];
        }
      } catch (error) {
        console.error('[useTrendingAgents] Fetch error:', error);
        // Return empty array on error
        return [];
      }
    },
    staleTime: 5 * 60 * 1000, // 5 minutes (matches API cache)
    gcTime: 10 * 60 * 1000, // 10 minutes
    refetchOnWindowFocus: false,
  });
}

/**
 * Hook to increment view count for an agent and update caches
 */
export function useIncrementViewCount() {
  const queryClient = useQueryClient();

  const incrementViewCount = async (agentId: string) => {
    console.log(`🔍 Incrementing view count for agent: ${agentId}`);
    
    // Extract the actual UUID from the composite ID (format: "name-version")
    // For optimized agents, we need to find the actual UUID
    // First, update all query caches optimistically
    
    // Update latest agents cache
    queryClient.setQueriesData(
      { queryKey: ["agents", "latest"] },
      (oldData: Agent[] | undefined) => {
        if (!oldData) return oldData;
        return oldData.map(agent => 
          agent.id === agentId 
            ? { ...agent, view_count: agent.view_count + 1 }
            : agent
        );
      }
    );
    
    // Update trending agents cache
    queryClient.setQueriesData(
      { queryKey: ["agents", "trending"] },
      (oldData: Agent[] | undefined) => {
        if (!oldData) return oldData;
        return oldData.map(agent => 
          agent.id === agentId 
            ? { ...agent, view_count: agent.view_count + 1 }
            : agent
        );
      }
    );
    
    // Update the regular agents cache as well
    queryClient.setQueryData(["agents", "all"], (oldData: Agent[] = []) => 
      oldData.map(agent => 
        agent.id === agentId 
          ? { ...agent, view_count: agent.view_count + 1 }
          : agent
      )
    );

    // For optimized agents, we do not have the real UUID, so we cannot call the database function
    // The view count increment will happen when the user opens the full agent page
    // This is just for optimistic UI updates
    console.log(`✅ View count updated optimistically for agent: ${agentId}`);
  };

  return { incrementViewCount };
}
