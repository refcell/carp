import { Agent } from '@/hooks/useAgents';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Badge } from '@/components/ui/badge';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { Button } from '@/components/ui/button';
import { Eye, Calendar, Copy } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { useToast } from '@/hooks/use-toast';

interface TrendingModalProps {
  agent: Agent | null;
  open: boolean;
  onClose: () => void;
}

export function TrendingModal({ agent, open, onClose }: TrendingModalProps) {
  const { toast } = useToast();

  if (!agent) return null;

  const copyDefinition = () => {
    const definition = typeof agent.definition === 'string' 
      ? agent.definition 
      : JSON.stringify(agent.definition, null, 2);
    
    navigator.clipboard.writeText(definition);
    toast({
      title: "Copied!",
      description: "Agent definition copied to clipboard"
    });
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="text-2xl font-bold">{agent.name}</DialogTitle>
        </DialogHeader>
        
        <div className="space-y-6">
          {/* Agent Info */}
          <div>
            <p className="text-lg text-muted-foreground leading-relaxed">
              {agent.description}
            </p>
          </div>

          {/* Author Info */}
          {agent.profiles && (
            <div className="flex items-center space-x-3">
              <Avatar className="w-8 h-8">
                <AvatarImage src={agent.profiles.avatar_url || ''} />
                <AvatarFallback>
                  {agent.profiles.display_name?.[0] || agent.profiles.github_username?.[0] || 'U'}
                </AvatarFallback>
              </Avatar>
              <div>
                <p className="font-medium">
                  {agent.profiles.display_name || agent.profiles.github_username}
                </p>
                <p className="text-sm text-muted-foreground">
                  @{agent.profiles.github_username}
                </p>
              </div>
            </div>
          )}

          {/* Tags */}
          {agent.tags && agent.tags.length > 0 && (
            <div>
              <h4 className="font-medium mb-2">Tags</h4>
              <div className="flex flex-wrap gap-2">
                {agent.tags.map((tag, index) => (
                  <Badge key={index} variant="secondary">
                    {tag}
                  </Badge>
                ))}
              </div>
            </div>
          )}

          {/* Stats */}
          <div className="flex items-center space-x-6 text-sm text-muted-foreground">
            <div className="flex items-center space-x-2">
              <Eye className="w-4 h-4" />
              <span>{agent.view_count} views</span>
            </div>
            <div className="flex items-center space-x-2">
              <Calendar className="w-4 h-4" />
              <span>Created {formatDistanceToNow(new Date(agent.created_at), { addSuffix: true })}</span>
            </div>
          </div>

          {/* Definition */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <h4 className="font-medium">Agent Definition</h4>
              <Button
                variant="outline"
                size="sm"
                onClick={copyDefinition}
                className="flex items-center space-x-2"
              >
                <Copy className="w-3 h-3" />
                <span>Copy</span>
              </Button>
            </div>
            <div className="bg-muted rounded-lg p-4 max-h-64 overflow-y-auto">
              <pre className="text-sm font-mono whitespace-pre-wrap">
                {typeof agent.definition === 'string' 
                  ? agent.definition 
                  : JSON.stringify(agent.definition, null, 2)
                }
              </pre>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}