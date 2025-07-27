import { useState } from 'react';
import { useAgents } from '@/hooks/useAgents';
import { SearchBar } from '@/components/SearchBar';
import { AgentCard } from '@/components/AgentCard';
import { TrendingModal } from '@/components/TrendingModal';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { TrendingUp, Clock, Star, Sparkles } from 'lucide-react';
import { Agent } from '@/hooks/useAgents';

const Index = () => {
  const { 
    agents, 
    loading, 
    searchQuery, 
    setSearchQuery, 
    trendingAgents, 
    latestAgents, 
    topAgents,
    incrementViewCount 
  } = useAgents();
  
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
  const [showTrendingModal, setShowTrendingModal] = useState(false);

  const featuredAgent = trendingAgents[0];

  const handleAgentClick = async (agent: Agent) => {
    await incrementViewCount(agent.id);
    setSelectedAgent(agent);
    setShowTrendingModal(true);
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Hero Section */}
      <section className="py-16 px-4">
        <div className="container mx-auto text-center">
          <div className="max-w-4xl mx-auto">
            <h1 className="text-4xl md:text-6xl font-bold mb-6 bg-gradient-to-r from-primary to-accent bg-clip-text text-transparent">
              Claude Agent Registry
            </h1>
            <p className="text-xl text-muted-foreground mb-8 max-w-2xl mx-auto">
              Discover, share, and use powerful Claude AI agents created by the community. 
              Find the perfect assistant for any task.
            </p>
            
            {/* Search Bar */}
            <div className="mb-12">
              <SearchBar 
                value={searchQuery}
                onChange={setSearchQuery}
                placeholder="Search for agents, tags, or descriptions..."
              />
            </div>
          </div>
        </div>
      </section>

      {/* Main Content - No Scrolling Layout */}
      <section className="px-4 pb-16">
        <div className="container mx-auto">
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8 max-w-7xl mx-auto">
            
            {/* Featured/Trending Agent - Elevated */}
            <div className="lg:col-span-3 mb-8">
              {featuredAgent ? (
                <Card className="relative overflow-hidden bg-gradient-to-br from-primary/5 to-accent/5 border-primary/20 shadow-lg">
                  <div className="absolute top-4 right-4">
                    <div className="flex items-center space-x-1 bg-primary/10 text-primary px-3 py-1 rounded-full text-sm font-medium">
                      <Sparkles className="w-4 h-4" />
                      <span>Trending</span>
                    </div>
                  </div>
                  <CardContent className="p-8">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6 items-center">
                      <div>
                        <h2 className="text-3xl font-bold mb-4">{featuredAgent.name}</h2>
                        <p className="text-lg text-muted-foreground mb-6 line-clamp-3">
                          {featuredAgent.description}
                        </p>
                        <div className="flex items-center space-x-4 text-sm text-muted-foreground mb-6">
                          <div className="flex items-center space-x-1">
                            <TrendingUp className="w-4 h-4" />
                            <span>{featuredAgent.view_count} views</span>
                          </div>
                          {featuredAgent.profiles && (
                            <span>by {featuredAgent.profiles.display_name || featuredAgent.profiles.github_username}</span>
                          )}
                        </div>
                        <Button 
                          size="lg" 
                          onClick={() => handleAgentClick(featuredAgent)}
                          className="bg-gradient-to-r from-primary to-accent hover:from-primary/90 hover:to-accent/90"
                        >
                          View Agent Details
                        </Button>
                      </div>
                      <div className="hidden md:block">
                        <div className="bg-muted/50 rounded-lg p-4 h-48 flex items-center justify-center">
                          <div className="text-center text-muted-foreground">
                            <Sparkles className="w-12 h-12 mx-auto mb-2 opacity-50" />
                            <p className="text-sm">Featured Agent</p>
                          </div>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ) : (
                <Card className="bg-muted/20">
                  <CardContent className="p-8 text-center">
                    <TrendingUp className="w-12 h-12 mx-auto mb-4 opacity-50" />
                    <p className="text-muted-foreground">No trending agents yet. Be the first to create one!</p>
                  </CardContent>
                </Card>
              )}
            </div>

            {/* Latest Agents Column */}
            <div>
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center space-x-2">
                    <Clock className="w-5 h-5" />
                    <span>Latest</span>
                  </CardTitle>
                  <CardDescription>Recently published agents</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {loading ? (
                    <div className="space-y-4">
                      {Array.from({ length: 3 }).map((_, i) => (
                        <div key={i} className="h-20 bg-muted rounded animate-pulse" />
                      ))}
                    </div>
                  ) : latestAgents.length > 0 ? (
                    <div className="space-y-4 max-h-96 overflow-y-auto">
                      {latestAgents.map((agent) => (
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

            {/* Top Agents Column */}
            <div>
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center space-x-2">
                    <Star className="w-5 h-5" />
                    <span>Top Rated</span>
                  </CardTitle>
                  <CardDescription>Most viewed agents</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {loading ? (
                    <div className="space-y-4">
                      {Array.from({ length: 3 }).map((_, i) => (
                        <div key={i} className="h-20 bg-muted rounded animate-pulse" />
                      ))}
                    </div>
                  ) : topAgents.length > 0 ? (
                    <div className="space-y-4 max-h-96 overflow-y-auto">
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
            </div>

            {/* Search Results or Welcome */}
            <div>
              <Card>
                <CardHeader>
                  <CardTitle>
                    {searchQuery ? `Search Results` : 'All Agents'}
                  </CardTitle>
                  <CardDescription>
                    {searchQuery ? `Results for "${searchQuery}"` : 'Browse all available agents'}
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {loading ? (
                    <div className="space-y-4">
                      {Array.from({ length: 3 }).map((_, i) => (
                        <div key={i} className="h-20 bg-muted rounded animate-pulse" />
                      ))}
                    </div>
                  ) : agents.length > 0 ? (
                    <div className="space-y-4 max-h-96 overflow-y-auto">
                      {agents.slice(0, 10).map((agent) => (
                        <div key={agent.id} className="cursor-pointer" onClick={() => handleAgentClick(agent)}>
                          <AgentCard agent={agent} />
                        </div>
                      ))}
                    </div>
                  ) : (
                    <p className="text-muted-foreground text-center py-8">
                      {searchQuery ? 'No agents match your search' : 'No agents found'}
                    </p>
                  )}
                </CardContent>
              </Card>
            </div>
          </div>
        </div>
      </section>

      {/* Trending Modal */}
      <TrendingModal
        agent={selectedAgent}
        open={showTrendingModal}
        onClose={() => {
          setShowTrendingModal(false);
          setSelectedAgent(null);
        }}
      />
    </div>
  );
};

export default Index;
