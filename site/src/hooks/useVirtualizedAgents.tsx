import { useMemo } from 'react';
import { Agent } from './useAgents';

interface VirtualizedAgentsOptions {
  itemHeight: number;
  containerHeight: number;
  overscan?: number;
}

export function useVirtualizedAgents(
  agents: Agent[],
  scrollTop: number,
  { itemHeight, containerHeight, overscan = 5 }: VirtualizedAgentsOptions
) {
  return useMemo(() => {
    const visibleCount = Math.ceil(containerHeight / itemHeight);
    const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - overscan);
    const endIndex = Math.min(agents.length - 1, startIndex + visibleCount + overscan * 2);
    
    const visibleAgents = agents.slice(startIndex, endIndex + 1);
    const totalHeight = agents.length * itemHeight;
    const offsetY = startIndex * itemHeight;

    return {
      visibleAgents,
      totalHeight,
      offsetY,
      startIndex,
      endIndex
    };
  }, [agents, scrollTop, itemHeight, containerHeight, overscan]);
}

// Hook for efficient search with debouncing and caching
export function useSearchCache() {
  const cache = useMemo(() => new Map<string, Agent[]>(), []);
  
  const getCachedResults = (query: string): Agent[] | undefined => {
    return cache.get(query.toLowerCase());
  };
  
  const setCachedResults = (query: string, results: Agent[]) => {
    // Limit cache size to prevent memory issues
    if (cache.size > 50) {
      const firstKey = cache.keys().next().value;
      cache.delete(firstKey);
    }
    cache.set(query.toLowerCase(), results);
  };
  
  return { getCachedResults, setCachedResults };
}