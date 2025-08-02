import { memo } from 'react';
import { Agent } from '@/hooks/useOptimizedAgents';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Eye } from 'lucide-react';

interface CompactAgentCardProps {
  agent: Agent;
  onClick?: () => void;
}

const CompactAgentCard = memo(function CompactAgentCard({ agent, onClick }: CompactAgentCardProps) {
  return (
    <Card 
      className="cursor-pointer hover:shadow-sm transition-all duration-200 h-auto"
      onClick={onClick}
    >
      <CardContent className="p-3">
        <h3 className="font-medium text-sm line-clamp-1 mb-1">
          {agent.name}
        </h3>
        <p className="text-xs text-muted-foreground line-clamp-2 mb-2">
          {agent.description}
        </p>
        
        {agent.tags && agent.tags.length > 0 && (
          <div className="flex flex-wrap gap-1 mb-2">
            {agent.tags.slice(0, 2).map((tag, index) => (
              <Badge key={index} variant="secondary" className="text-xs px-1.5 py-0">
                {tag}
              </Badge>
            ))}
          </div>
        )}
        
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <div className="flex items-center space-x-1">
            <Eye className="w-3 h-3" />
            <span>{agent.view_count}</span>
          </div>
          <span className="truncate">
            {agent.profiles?.display_name || agent.profiles?.github_username || 'Unknown'}
          </span>
        </div>
      </CardContent>
    </Card>
  );
});

export { CompactAgentCard };