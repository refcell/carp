import { useEffect, useRef, useState } from 'react';
import { Agent } from '@/hooks/useAgents';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Badge } from '@/components/ui/badge';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { Button } from '@/components/ui/button';
import { Eye, Calendar, Copy, ChevronDown, ChevronUp } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import { useToast } from '@/hooks/use-toast';

interface TrendingModalProps {
  agent: Agent | null;
  open: boolean;
  onClose: () => void;
  onViewIncrement?: (agentId: string) => void;
}

export function TrendingModal({ agent, open, onClose, onViewIncrement }: TrendingModalProps) {
  const { toast } = useToast();
  const hasIncrementedRef = useRef(false);
  const [isDescriptionExpanded, setIsDescriptionExpanded] = useState(false);

  // Track when modal opens and increment view count
  useEffect(() => {
    if (open && agent && onViewIncrement && !hasIncrementedRef.current) {
      onViewIncrement(agent.id);
      hasIncrementedRef.current = true;
    }
    
    // Reset when modal closes
    if (!open) {
      hasIncrementedRef.current = false;
      setIsDescriptionExpanded(false);
    }
  }, [open, agent, onViewIncrement]);

  if (!agent) return null;

  const DESCRIPTION_CHAR_LIMIT = 150;
  const shouldTruncateDescription = agent.description.length > DESCRIPTION_CHAR_LIMIT;
  const displayDescription = shouldTruncateDescription && !isDescriptionExpanded
    ? `${agent.description.slice(0, DESCRIPTION_CHAR_LIMIT)}...`
    : agent.description;

  // Extract prompt from agent definition
  const getPromptFromDefinition = () => {
    if (typeof agent.definition === 'string') {
      return agent.definition;
    }
    
    if (typeof agent.definition === 'object' && agent.definition?.prompt) {
      return agent.definition.prompt;
    }
    
    return null;
  };

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

  const copyPrompt = () => {
    const prompt = getPromptFromDefinition();
    if (prompt) {
      navigator.clipboard.writeText(prompt);
      toast({
        title: "Copied!",
        description: "Agent definition copied to clipboard"
      });
    }
  };

  const prompt = getPromptFromDefinition();

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="text-2xl font-bold">{agent.name}</DialogTitle>
        </DialogHeader>
        
        <div className="space-y-6">
          {/* Agent Description with collapsible functionality */}
          <div>
            <p className="text-lg text-muted-foreground leading-relaxed">
              {displayDescription}
            </p>
            {shouldTruncateDescription && (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setIsDescriptionExpanded(!isDescriptionExpanded)}
                className="mt-2 p-0 h-auto text-sm text-primary hover:bg-transparent"
                aria-expanded={isDescriptionExpanded}
                aria-label={isDescriptionExpanded ? "Show less description" : "Show more description"}
              >
                {isDescriptionExpanded ? (
                  <>
                    <ChevronUp className="w-4 h-4 mr-1" />
                    Read less
                  </>
                ) : (
                  <>
                    <ChevronDown className="w-4 h-4 mr-1" />
                    Read more
                  </>
                )}
              </Button>
            )}
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

          {/* Agent Definition - User-friendly display */}
          {prompt && (
            <div>
              <div className="flex items-center justify-between mb-2">
                <h4 className="font-medium">Agent Definition</h4>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={copyPrompt}
                  className="flex items-center space-x-2"
                  aria-label="Copy agent definition"
                >
                  <Copy className="w-3 h-3" />
                  <span>Copy</span>
                </Button>
              </div>
              <div className="bg-muted rounded-lg p-4 max-h-64 overflow-y-auto">
                <p className="text-sm leading-relaxed whitespace-pre-wrap">
                  {prompt}
                </p>
              </div>
            </div>
          )}

          {/* Full Definition - Only show if it contains more than just prompt */}
          {typeof agent.definition === 'object' && agent.definition && Object.keys(agent.definition).length > 1 && (
            <div>
              <div className="flex items-center justify-between mb-2">
                <h4 className="font-medium">Full Agent Definition</h4>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={copyDefinition}
                  className="flex items-center space-x-2"
                  aria-label="Copy full agent definition"
                >
                  <Copy className="w-3 h-3" />
                  <span>Copy JSON</span>
                </Button>
              </div>
              <div className="bg-muted rounded-lg p-4 max-h-64 overflow-y-auto">
                <pre className="text-sm font-mono whitespace-pre-wrap">
                  {JSON.stringify(agent.definition, null, 2)}
                </pre>
              </div>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}