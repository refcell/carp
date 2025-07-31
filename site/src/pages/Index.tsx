import { useState, useEffect } from 'react';
import { useAgents } from '@/hooks/useAgents';
import { SearchBar } from '@/components/SearchBar';
import { AgentCard } from '@/components/AgentCard';
import { TrendingModal } from '@/components/TrendingModal';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { TrendingUp, Clock, Star, Sparkles, Search, BarChart3, Users, Trophy } from 'lucide-react';
import { Agent } from '@/hooks/useAgents';
import { supabase } from '@/integrations/supabase/client';

interface UserStats {
  github_username: string | null;
  display_name: string | null;
  avatar_url: string | null;
  agent_count: number;
}

const Index = () => {
  const {
    agents,
    loading,
    searchLoading,
    searchQuery,
    setSearchQuery,
    trendingAgents,
    latestAgents,
    topAgents,
    incrementViewCount
  } = useAgents();

  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
  const [showTrendingModal, setShowTrendingModal] = useState(false);
  const [totalAgentCount, setTotalAgentCount] = useState(0);
  const [userLeaderboard, setUserLeaderboard] = useState<UserStats[]>([]);
  const [loadingStats, setLoadingStats] = useState(true);

  const featuredAgent = trendingAgents[0];

  const handleAgentClick = (agent: Agent) => {
    setSelectedAgent(agent);
    setShowTrendingModal(true);
  };

  // Fetch total agent count and user leaderboard
  useEffect(() => {
    const fetchStats = async () => {
      try {
        // Get total agent count
        const { count } = await supabase
          .from('agents')
          .select('*', { count: 'exact', head: true })
          .eq('is_public', true);

        setTotalAgentCount(count || 0);

        // Get user leaderboard
        const { data: leaderboardData, error } = await supabase
          .from('profiles')
          .select(`
            github_username,
            display_name,
            avatar_url,
            user_id
          `);

        if (leaderboardData && !error) {
          // Count agents for each user
          const userStatsPromises = leaderboardData.map(async (profile) => {
            const { count: agentCount } = await supabase
              .from('agents')
              .select('*', { count: 'exact', head: true })
              .eq('user_id', profile.user_id)
              .eq('is_public', true);

            return {
              github_username: profile.github_username,
              display_name: profile.display_name,
              avatar_url: profile.avatar_url,
              agent_count: agentCount || 0
            };
          });

          const userStats = await Promise.all(userStatsPromises);
          // Sort by agent count and take top 10
          const sortedStats = userStats
            .filter(stat => stat.agent_count > 0)
            .sort((a, b) => b.agent_count - a.agent_count)
            .slice(0, 10);

          setUserLeaderboard(sortedStats);
        }
      } catch (error) {
        console.error('Error fetching stats:', error);
      } finally {
        setLoadingStats(false);
      }
    };

    fetchStats();
  }, []);

  return (
          <div className="grid px-8 flex-grow mx-auto mb-8 mt-12 grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 lg:gap-6 lg:h-[calc(100vh-200px)]">
            {/* Left Column - Stats and Leaderboard */}
            <div className="md:col-span-1 lg:col-span-1 flex flex-col gap-3 lg:gap-4 order-3 md:order-2 lg:order-1 lg:min-h-0 lg:h-full">
              {/* Stats Cards Row */}
              <div className="grid grid-cols-2 gap-2 lg:gap-3">
                {/* Agent Count Card */}
                <Card className="bg-gradient-to-br from-primary/10 to-accent/10 border-muted/30">
                  <CardContent className="p-2 lg:p-3">
                    <div className="text-center">
                      <BarChart3 className="w-4 h-4 lg:w-5 lg:h-5 mx-auto mb-1 text-primary" />
                      <p className="text-base lg:text-lg font-bold">{totalAgentCount}</p>
                      <p className="text-xs text-muted-foreground">Agents</p>
                    </div>
                  </CardContent>
                </Card>

                {/* Active Users Card */}
                <Card className="bg-gradient-to-br from-accent/10 to-primary/10 border-muted/30">
                  <CardContent className="p-2 lg:p-3">
                    <div className="text-center">
                      <Users className="w-4 h-4 lg:w-5 lg:h-5 mx-auto mb-1 text-primary" />
                      <p className="text-base lg:text-lg font-bold">{userLeaderboard.length}</p>
                      <p className="text-xs text-muted-foreground">Users</p>
                    </div>
                  </CardContent>
                </Card>
              </div>

              {/* Leaderboard */}
              <Card className="flex-1 flex flex-col lg:min-h-0 lg:overflow-hidden">
                <CardHeader>
                  <CardTitle className="flex items-center space-x-2">
                    <Trophy className="w-5 h-5" />
                    <span>Leaderboard</span>
                  </CardTitle>
                  <CardDescription>Top contributors</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4 flex-1 flex flex-col lg:min-h-0">
                  {loadingStats ? (
                    <div className="space-y-4">
                      {Array.from({ length: 3 }).map((_, i) => (
                        <div key={i} className="h-12 bg-muted rounded animate-pulse" />
                      ))}
                    </div>
                  ) : userLeaderboard.length > 0 ? (
                    <div className="space-y-3 flex-1 overflow-y-auto lg:min-h-0">
                      {userLeaderboard.slice(0, 8).map((user, index) => (
                        <div key={`${user.github_username}-${index}`} className="flex items-center justify-between p-2 rounded-lg hover:bg-muted/50 transition-colors">
                          <div className="flex items-center space-x-3">
                            <div className="flex items-center justify-center w-6 h-6 rounded-full bg-primary/10 text-primary text-sm font-bold">
                              {index + 1}
                            </div>
                            <div className="flex-1 min-w-0">
                              <p className="text-sm font-medium truncate">
                                {user.display_name || user.github_username}
                              </p>
                            </div>
                          </div>
                          <div className="text-sm text-muted-foreground font-medium">
                            {user.agent_count}
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p className="text-muted-foreground text-center py-8">No users found</p>
                  )}
                </CardContent>
              </Card>
            </div>

            {/* Center Column - Search Bar and Content */}
            <div className="md:col-span-2 lg:col-span-2 flex flex-col gap-4 lg:gap-6 order-1 md:order-1 lg:order-2 lg:min-h-0 lg:h-full">
              {/* Search Bar - Always Visible */}
              <div className="flex justify-center flex-shrink-0">
                <div className="w-full max-w-2xl">
                  <SearchBar
                    value={searchQuery}
                    onChange={setSearchQuery}
                    placeholder="Search for agents, tags, or descriptions..."
                  />
                </div>
              </div>

              {searchQuery ? (
                /* Search Results */
                <div className="flex-1 flex flex-col lg:min-h-0 lg:overflow-hidden">
                  <div className="mb-4 flex-shrink-0">
                    <h2 className="text-xl font-bold mb-1 text-center">Search Results</h2>
                    <p className="text-muted-foreground text-center text-sm">
                      {searchLoading ? 'Searching...' : `${agents.length} result${agents.length !== 1 ? 's' : ''} for "${searchQuery}"`}
                    </p>
                  </div>

                  <div className="flex-1 overflow-y-auto pr-2 lg:min-h-0">
                    {searchLoading ? (
                      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                        {Array.from({ length: 6 }).map((_, i) => (
                          <div key={i} className="h-32 bg-muted rounded-lg animate-pulse" />
                        ))}
                      </div>
                    ) : agents.length > 0 ? (
                      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                        {agents.map((agent) => (
                          <div key={agent.id} className="cursor-pointer" onClick={() => handleAgentClick(agent)}>
                            <AgentCard agent={agent} />
                          </div>
                        ))}
                      </div>
                    ) : (
                      <div className="text-center py-16">
                        <div className="mx-auto w-16 h-16 bg-muted rounded-full flex items-center justify-center mb-4">
                          <Search className="w-6 h-6 text-muted-foreground" />
                        </div>
                        <h3 className="text-lg font-semibold mb-2">No agents found</h3>
                        <p className="text-muted-foreground text-sm">
                          No agents match your search for "{searchQuery}". Try different keywords.
                        </p>
                      </div>
                    )}
                  </div>
                </div>
              ) : (
                /* Dashboard Content - Featured Agent and Top Agents */
                <>
                  {/* Featured Agent - Smaller */}
                  {featuredAgent ? (
                    <Card className="relative overflow-hidden bg-gradient-to-br from-primary/5 to-accent/5 border-primary/20 shadow-lg">
                      <div className="absolute top-3 right-3">
                        <div className="flex items-center space-x-1 bg-primary/10 text-primary px-2 py-1 rounded-full text-xs font-medium">
                          <Sparkles className="w-3 h-3" />
                          <span>Featured</span>
                        </div>
                      </div>
                      <CardContent className="p-4">
                        <div className="flex items-center gap-4">
                          <div className="flex-1">
                            <h2 className="text-xl font-bold mb-2">{featuredAgent.name}</h2>
                            <p className="text-sm text-muted-foreground mb-3 line-clamp-2">
                              {featuredAgent.description}
                            </p>
                            <div className="flex items-center space-x-4 text-xs text-muted-foreground mb-3">
                              <div className="flex items-center space-x-1">
                                <TrendingUp className="w-3 h-3" />
                                <span>{featuredAgent.view_count} views</span>
                              </div>
                              {featuredAgent.profiles && (
                                <span>by {featuredAgent.profiles.display_name || featuredAgent.profiles.github_username}</span>
                              )}
                            </div>
                          </div>
                          <Button
                            onClick={() => handleAgentClick(featuredAgent)}
                            className="bg-gradient-to-r from-primary to-accent hover:from-primary/90 hover:to-accent/90"
                          >
                            View Details
                          </Button>
                        </div>
                      </CardContent>
                    </Card>
                  ) : (
                    <Card className="bg-muted/20">
                      <CardContent className="p-4 text-center">
                        <TrendingUp className="w-8 h-8 mx-auto mb-2 opacity-50" />
                        <p className="text-muted-foreground text-sm">No trending agents yet. Be the first to create one!</p>
                      </CardContent>
                    </Card>
                  )}

                  {/* Top Agents - Takes remaining space */}
                  <Card className="flex-1 flex flex-col lg:min-h-0 lg:overflow-hidden">
                    <CardHeader className="flex-shrink-0">
                      <CardTitle className="flex items-center space-x-2">
                        <Star className="w-5 h-5" />
                        <span>Top Rated</span>
                      </CardTitle>
                      <CardDescription>Most viewed agents</CardDescription>
                    </CardHeader>
                    <CardContent className="flex-1 lg:min-h-0 lg:overflow-hidden">
                      {loading ? (
                        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                          {Array.from({ length: 6 }).map((_, i) => (
                            <div key={i} className="h-20 bg-muted rounded animate-pulse" />
                          ))}
                        </div>
                      ) : topAgents.length > 0 ? (
                        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 lg:h-full lg:overflow-y-auto pr-2 lg:min-h-0">
                          {topAgents.map((agent) => (
                            <div key={agent.id} className="cursor-pointer" onClick={() => handleAgentClick(agent)}>
                              <AgentCard agent={agent} />
                            </div>
                          ))}
                        </div>
                      ) : (
                        <p className="text-muted-foreground text-center py-8">No agents found</p>
                      )}
                    </CardContent>
                  </Card>
                </>
              )}
            </div>

            {/* Right Column - Latest Agents */}
            <div className="md:col-span-1 lg:col-span-1 flex flex-col order-2 md:order-3 lg:order-3 lg:min-h-0 lg:h-full">
              <Card className="flex-1 flex flex-col lg:min-h-0 lg:overflow-hidden">
                <CardHeader>
                  <CardTitle className="flex items-center space-x-2">
                    <Clock className="w-5 h-5" />
                    <span>Latest</span>
                  </CardTitle>
                  <CardDescription>Recently published</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4 flex-1 flex flex-col lg:min-h-0">
                  {loading ? (
                    <div className="space-y-4">
                      {Array.from({ length: 3 }).map((_, i) => (
                        <div key={i} className="h-20 bg-muted rounded animate-pulse" />
                      ))}
                    </div>
                  ) : latestAgents.length > 0 ? (
                    <div className="space-y-4 flex-1 overflow-y-auto max-h-96 lg:max-h-none lg:min-h-0">
                      {latestAgents.slice(0, 6).map((agent) => (
                        <div key={agent.id} className="cursor-pointer" onClick={() => handleAgentClick(agent)}>
                          <AgentCard agent={agent} />
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p className="text-muted-foreground text-center py-8">No agents found</p>
                  )}
                </CardContent>
              </Card>
            </div>
          {/* Trending Modal */}
          <TrendingModal
            agent={selectedAgent}
            open={showTrendingModal}
            onClose={() => {
              setShowTrendingModal(false);
              setSelectedAgent(null);
            }}
            onViewIncrement={incrementViewCount}
          />
        </div>
  );
};

export default Index;
