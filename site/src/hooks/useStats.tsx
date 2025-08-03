import { useQuery } from '@tanstack/react-query';
import { supabase } from '@/integrations/supabase/client';

interface ProfileData {
  github_username: string | null;
  display_name: string | null;
  avatar_url: string | null;
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

      // Use the new leaderboard function
      const { data: leaderboardData, error } = await supabase
        .rpc('get_user_leaderboard');

      if (error) {
        console.error('Error fetching leaderboard:', error);
        return { 
          totalAgentCount: agentCount || 0, 
          userLeaderboard: [], 
          totalUserCount: userCount || 0 
        };
      }

      // Map the leaderboard data to UserStats format
      const userStats: UserStats[] = (leaderboardData || []).map((user: any) => ({
        github_username: user.github_username,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        agent_count: user.agent_count
      }));

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