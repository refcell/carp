import { useState, useEffect, useCallback } from 'react';
import { useAgents, Agent } from '@/hooks/useAgents';
import { AgentCard } from '@/components/AgentCard';
import { TrendingModal } from '@/components/TrendingModal';
import { Button } from '@/components/ui/button';
import { Loader2, Search } from 'lucide-react';

const AllAgents = () => {
  const { fetchAgentsPaginated, incrementViewCount } = useAgents();
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
  const [showTrendingModal, setShowTrendingModal] = useState(false);

  const loadAgents = useCallback(async (offset: number = 0, append: boolean = false) => {
    if (offset === 0) {
      setLoading(true);
    } else {
      setLoadingMore(true);
    }

    try {
      const result = await fetchAgentsPaginated(20, offset);

      if (append) {
        setAgents(prevAgents => {
          // Prevent duplicates by filtering out agents that already exist
          const existingIds = new Set(prevAgents.map(agent => agent.id));
          const newAgents = result.agents.filter(agent => !existingIds.has(agent.id));
          return [...prevAgents, ...newAgents];
        });
      } else {
        setAgents(result.agents);
      }

      setHasMore(result.hasMore);
    } catch (error) {
      console.error('Error loading agents:', error);
    } finally {
      if (offset === 0) {
        setLoading(false);
      } else {
        setLoadingMore(false);
      }
    }
  }, [fetchAgentsPaginated]);

  const handleLoadMore = () => {
    if (!loadingMore && hasMore) {
      loadAgents(agents.length, true);
    }
  };

  const handleAgentClick = (agent: Agent) => {
    setSelectedAgent(agent);
    setShowTrendingModal(true);
  };

  useEffect(() => {
    loadAgents();
  }, [loadAgents]);

  return (
    <div className="min-h-screen bg-background">
      {/* Page Header */}
      <section className="pt-16 pb-8">
        <div className="container mx-auto">
          <div className="max-w-4xl mx-auto text-center">
            <h1 className="text-4xl pb-2 md:text-5xl font-bold mb-4 bg-gradient-to-r from-primary to-accent bg-clip-text text-transparent">
              All Agents
            </h1>
            <p className="text-xl text-muted-foreground mb-8 max-w-2xl mx-auto">
              Browse all Claude AI agents created by the community. Discover the perfect assistant for any task.
            </p>
          </div>
        </div>
      </section>

      {/* Agents Grid */}
      <section className="container mx-auto pb-16">
        <div className="max-w-7xl mx-auto">
          {loading ? (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
              {Array.from({ length: 20 }).map((_, i) => (
                <div key={i} className="h-64 bg-muted rounded-lg animate-pulse" />
              ))}
            </div>
          ) : agents.length > 0 ? (
            <>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6 mb-12">
                {agents.map((agent) => (
                  <div key={agent.id} className="cursor-pointer" onClick={() => handleAgentClick(agent)}>
                    <AgentCard agent={agent} />
                  </div>
                ))}
              </div>

              {/* Load More Button */}
              {hasMore && (
                <div className="text-center">
                  <Button
                    onClick={handleLoadMore}
                    disabled={loadingMore}
                    size="lg"
                    variant="outline"
                    className="min-w-32"
                  >
                    {loadingMore ? (
                      <>
                        <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                        Loading...
                      </>
                    ) : (
                      'Show More'
                    )}
                  </Button>
                </div>
              )}

              {/* End of results message */}
              {!hasMore && agents.length > 20 && (
                <div className="text-center pt-8">
                  <p className="text-muted-foreground">
                    You've reached the end of all available agents.
                  </p>
                </div>
              )}
            </>
          ) : (
            <div className="text-center py-16">
              <div className="mx-auto w-24 h-24 bg-muted rounded-full flex items-center justify-center mb-4">
                <Search className="w-8 h-8 text-muted-foreground" />
              </div>
              <h3 className="text-lg font-semibold mb-2">No agents found</h3>
              <p className="text-muted-foreground">
                No agents have been published yet. Be the first to create one!
              </p>
            </div>
          )}
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
        onViewIncrement={incrementViewCount}
      />
    </div>
  );
};

export default AllAgents;
