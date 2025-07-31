import { useQuery } from '@tanstack/react-query';
import { supabase } from '@/integrations/supabase/client';

interface ProfileData {
  github_username: string | null;
  display_name: string | null;
  avatar_url: string | null;
}

interface LeaderboardItem {
  github_username: string | null;
  display_name: string | null;
  avatar_url: string | null;
  user_id: string;
  agents: { id: string }[];
}

export interface UserStats {
  github_username: string | null;
  display_name: string | null;
  avatar_url: string | null;
  agent_count: number;
}

export interface DashboardStats {
  totalAgentCount: number;
  userLeaderboard: UserStats[];
  totalUserCount: number;
}

export function useStats() {
  return useQuery({
    queryKey: ['dashboard-stats'],
    queryFn: async (): Promise<DashboardStats> => {
      // Get total agent count
      const { count: agentCount } = await supabase
        .from('agents')
        .select('*', { count: 'exact', head: true })
        .eq('is_public', true);

      // Get total user count  
      const { count: userCount } = await supabase
        .from('profiles')
        .select('*', { count: 'exact', head: true });

      // Optimized leaderboard query using aggregation
      const { data: leaderboardData, error } = await supabase
        .from('profiles')
        .select(`
          github_username,
          display_name,
          avatar_url,
          user_id,
          agents!inner(
            id
          )
        `)
        .eq('agents.is_public', true);

      if (error) {
        console.error('Error fetching leaderboard:', error);
        return { 
          totalAgentCount: agentCount || 0, 
          userLeaderboard: [], 
          totalUserCount: userCount || 0 
        };
      }

      // Count agents per user from the joined data
      const userAgentCounts = new Map<string, { profile: ProfileData, count: number }>();
      
      leaderboardData?.forEach((item: LeaderboardItem) => {
        const userId = item.user_id;
        if (!userAgentCounts.has(userId)) {
          userAgentCounts.set(userId, {
            profile: {
              github_username: item.github_username,
              display_name: item.display_name,
              avatar_url: item.avatar_url
            },
            count: 0
          });
        }
        userAgentCounts.get(userId)!.count++;
      });

      // Convert to array and sort
      const userStats: UserStats[] = Array.from(userAgentCounts.values())
        .map(({ profile, count }) => ({
          github_username: profile.github_username,
          display_name: profile.display_name,
          avatar_url: profile.avatar_url,
          agent_count: count
        }))
        .filter(stat => stat.agent_count > 0)
        .sort((a, b) => b.agent_count - a.agent_count)
        .slice(0, 10);

      return {
        totalAgentCount: agentCount || 0,
        userLeaderboard: userStats,
        totalUserCount: userCount || 0
      };
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 10 * 60 * 1000, // 10 minutes
    refetchOnWindowFocus: false
  });
}