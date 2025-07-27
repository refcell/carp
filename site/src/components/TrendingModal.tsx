import { Agent } from '@/hooks/useAgents';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Badge } from '@/components/ui/badge';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { Eye, Calendar, Code, X } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { Button } from '@/components/ui/button';

interface TrendingModalProps {
  agent: Agent | null;
  open: boolean;
  onClose: () => void;
}

export function TrendingModal({ agent, open, onClose }: TrendingModalProps) {
  if (!agent) return null;

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader className="flex flex-row items-start justify-between space-y-0">
          <div className="flex-1">
            <DialogTitle className="text-2xl font-bold mb-2">{agent.name}</DialogTitle>
            <p className="text-muted-foreground">{agent.description}</p>
          </div>
          <Button variant="ghost" size="sm" onClick={onClose}>
            <X className="w-4 h-4" />
          </Button>
        </DialogHeader>
        
        <div className="space-y-6">
          {/* Author info */}
          {agent.profiles && (
            <div className="flex items-center space-x-3">
              <Avatar className="w-10 h-10">
                <AvatarImage src={agent.profiles.avatar_url || ''} />
                <AvatarFallback>
                  {agent.profiles.display_name?.[0] || agent.profiles.github_username?.[0] || 'U'}
                </AvatarFallback>
              </Avatar>
              <div>
                <p className="font-medium">
                  {agent.profiles.display_name || agent.profiles.github_username}
                </p>
                <div className="flex items-center space-x-4 text-sm text-muted-foreground">
                  <div className="flex items-center space-x-1">
                    <Eye className="w-3 h-3" />
                    <span>{agent.view_count} views</span>
                  </div>
                  <div className="flex items-center space-x-1">
                    <Calendar className="w-3 h-3" />
                    <span>{formatDistanceToNow(new Date(agent.created_at), { addSuffix: true })}</span>
                  </div>
                </div>
              </div>
            </div>
          )}

          {/* Tags */}
          {agent.tags && agent.tags.length > 0 && (
            <div>
              <h3 className="font-semibold mb-2">Tags</h3>
              <div className="flex flex-wrap gap-2">
                {agent.tags.map((tag, index) => (
                  <Badge key={index} variant="secondary">
                    {tag}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          {/* Agent Definition */}
          <div>
            <h3 className="font-semibold mb-2 flex items-center">
              <Code className="w-4 h-4 mr-2" />
              Agent Definition
            </h3>
            <div className="bg-muted rounded-lg p-4">
              <pre className="text-sm overflow-x-auto whitespace-pre-wrap">
                {JSON.stringify(agent.definition, null, 2)}
              </pre>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}