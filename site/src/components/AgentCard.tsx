import { memo } from 'react';
import { Agent } from '@/hooks/useOptimizedAgents';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { Eye, Calendar } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

interface AgentCardProps {
  agent: Agent;
  onClick?: () => void;
  showAuthor?: boolean;
}

const AgentCard = memo(function AgentCard({ agent, onClick, showAuthor = true }: AgentCardProps) {
  return (
    <Card 
      className="cursor-pointer hover:shadow-md transition-all duration-200 flex flex-col"
      onClick={onClick}
    >
      <CardHeader className={`pb-3 ${!showAuthor ? 'pr-20' : ''}`}>
        <div className="flex items-start justify-between">
          <div className="flex-1 min-w-0">
            <CardTitle className="text-lg font-semibold truncate mb-2 flex items-center">
              {agent.name}
              {agent.definition?.version && (
                <span className="text-sm font-normal text-muted-foreground ml-2">v{agent.definition.version}</span>
              )}
            </CardTitle>
            <CardDescription className="line-clamp-2 mt-1 text-sm leading-relaxed">
              {agent.description}
            </CardDescription>
          </div>
        </div>
        
        {showAuthor && agent.profiles && (
          <div className="flex items-center space-x-2 mt-3">
            <Avatar className="w-6 h-6">
              <AvatarImage src={agent.profiles.avatar_url || ''} />
              <AvatarFallback>
                {(agent.profiles.display_name || agent.profiles.github_username || 'U')[0].toUpperCase()}
              </AvatarFallback>
            </Avatar>
            <span className="text-sm text-muted-foreground truncate">
              {agent.profiles.display_name || agent.profiles.github_username}
            </span>
          </div>
        )}
      </CardHeader>
      
      <CardContent className="pt-0 flex flex-col px-6 pb-4">
        <div>
          {agent.tags && agent.tags.length > 0 && (
            <div className="flex flex-wrap gap-1 mb-4">
              {agent.tags.slice(0, 3).map((tag, index) => (
                <Badge key={index} variant="secondary" className="text-xs px-2 py-1">
                  {tag}
                </Badge>
              ))}
              {agent.tags.length > 3 && (
                <Badge variant="secondary" className="text-xs px-2 py-1">
                  +{agent.tags.length - 3}
                </Badge>
              )}
            </div>
          )}
        </div>
        
        <div className="flex items-center justify-between text-sm text-muted-foreground mt-auto pt-4">
          <div className="flex items-center space-x-1">
            <Eye className="w-3 h-3" />
            <span>{agent.view_count}</span>
          </div>
          <div className="flex items-center space-x-1">
            <Calendar className="w-3 h-3" />
            <span>{formatDistanceToNow(new Date(agent.created_at), { addSuffix: true })}</span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
});

export { AgentCard };