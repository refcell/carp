import { useState, useEffect } from 'react';
import { useAuth } from '@/hooks/useAuth';
import { useAgents, Agent } from '@/hooks/useAgents';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { AgentCard } from '@/components/AgentCard';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Badge } from '@/components/ui/badge';
import { Edit, Trash2, Plus, X } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import { Link } from 'react-router-dom';

export default function Profile() {
  const { user } = useAuth();
  const { fetchUserAgents, updateAgent, deleteAgent } = useAgents();
  const { toast } = useToast();
  
  const [userAgents, setUserAgents] = useState<Agent[]>([]);
  const [editingAgent, setEditingAgent] = useState<Agent | null>(null);
  const [loading, setLoading] = useState(true);
  
  // Edit form state
  const [editName, setEditName] = useState('');
  const [editDescription, setEditDescription] = useState('');
  const [editDefinition, setEditDefinition] = useState('');
  const [editTags, setEditTags] = useState<string[]>([]);
  const [newTag, setNewTag] = useState('');

  useEffect(() => {
    if (user) {
      loadUserAgents();
    }
  }, [user]);

  const loadUserAgents = async () => {
    setLoading(true);
    const agents = await fetchUserAgents();
    setUserAgents(agents);
    setLoading(false);
  };

  const handleEdit = (agent: Agent) => {
    setEditingAgent(agent);
    setEditName(agent.name);
    setEditDescription(agent.description);
    setEditDefinition(JSON.stringify(agent.definition, null, 2));
    setEditTags(agent.tags || []);
  };

  const addTag = () => {
    if (newTag.trim() && !editTags.includes(newTag.trim())) {
      setEditTags([...editTags, newTag.trim()]);
      setNewTag('');
    }
  };

  const removeTag = (tagToRemove: string) => {
    setEditTags(editTags.filter(tag => tag !== tagToRemove));
  };

  const handleUpdate = async () => {
    if (!editingAgent) return;

    try {
      let parsedDefinition;
      try {
        parsedDefinition = JSON.parse(editDefinition);
      } catch {
        parsedDefinition = { prompt: editDefinition };
      }

      await updateAgent(editingAgent.id, {
        name: editName,
        description: editDescription,
        definition: parsedDefinition,
        tags: editTags.length > 0 ? editTags : null
      });

      toast({
        title: "Agent updated!",
        description: "Your agent has been successfully updated."
      });

      setEditingAgent(null);
      loadUserAgents();
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to update agent. Please try again.",
        variant: "destructive"
      });
    }
  };

  const handleDelete = async (agentId: string, agentName: string) => {
    if (!confirm(`Are you sure you want to delete "${agentName}"?`)) return;

    try {
      await deleteAgent(agentId);
      toast({
        title: "Agent deleted",
        description: "Your agent has been successfully deleted."
      });
      loadUserAgents();
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to delete agent. Please try again.",
        variant: "destructive"
      });
    }
  };

  if (!user) {
    return (
      <div className="container mx-auto px-4 py-8">
        <Card className="max-w-md mx-auto text-center">
          <CardContent className="pt-6">
            <p className="text-muted-foreground">Please sign in to view your profile</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-6xl mx-auto">
        {/* Profile Header */}
        <Card className="mb-8">
          <CardContent className="pt-6">
            <div className="flex items-center space-x-4">
              <Avatar className="w-20 h-20">
                <AvatarImage src={user.user_metadata?.avatar_url} />
                <AvatarFallback className="text-2xl">
                  {user.user_metadata?.user_name?.[0]?.toUpperCase() || 'U'}
                </AvatarFallback>
              </Avatar>
              <div>
                <h1 className="text-2xl font-bold">
                  {user.user_metadata?.full_name || user.user_metadata?.user_name}
                </h1>
                <p className="text-muted-foreground">@{user.user_metadata?.user_name}</p>
                <p className="text-sm text-muted-foreground mt-1">
                  {userAgents.length} agent{userAgents.length !== 1 ? 's' : ''} published
                </p>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Agents Section */}
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-xl font-semibold">My Agents</h2>
          <Button asChild>
            <Link to="/create">
              <Plus className="w-4 h-4 mr-2" />
              Create New Agent
            </Link>
          </Button>
        </div>

        {loading ? (
          <div className="text-center py-8">
            <p className="text-muted-foreground">Loading your agents...</p>
          </div>
        ) : userAgents.length === 0 ? (
          <Card>
            <CardContent className="text-center py-8">
              <p className="text-muted-foreground mb-4">You haven't created any agents yet</p>
              <Button asChild>
                <Link to="/create">Create Your First Agent</Link>
              </Button>
            </CardContent>
          </Card>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-3 2xl:grid-cols-4 gap-6">
            {userAgents.map((agent) => (
              <div key={agent.id} className="relative min-h-[220px] lg:min-h-[240px]">
                <AgentCard agent={agent} showAuthor={false} />
                <div className="absolute top-2 right-2 flex space-x-1 z-10">
                  <Button
                    size="sm"
                    variant="outline"
                    className="h-8 w-8 p-0 bg-background/80 backdrop-blur-sm border-2 hover:bg-background/90"
                    onClick={() => handleEdit(agent)}
                  >
                    <Edit className="w-3 h-3" />
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    className="h-8 w-8 p-0 bg-background/80 backdrop-blur-sm border-2 hover:bg-background/90"
                    onClick={() => handleDelete(agent.id, agent.name)}
                  >
                    <Trash2 className="w-3 h-3" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Edit Dialog */}
        <Dialog open={!!editingAgent} onOpenChange={() => setEditingAgent(null)}>
          <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
            <DialogHeader>
              <DialogTitle>Edit Agent</DialogTitle>
            </DialogHeader>
            
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-2">Name</label>
                <Input
                  value={editName}
                  onChange={(e) => setEditName(e.target.value)}
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-2">Description</label>
                <Textarea
                  value={editDescription}
                  onChange={(e) => setEditDescription(e.target.value)}
                  rows={3}
                />
              </div>

              <div>
                <label className="block text-sm font-medium mb-2">Tags</label>
                <div className="flex flex-wrap gap-2 mb-2">
                  {editTags.map((tag) => (
                    <Badge key={tag} variant="secondary" className="flex items-center gap-1">
                      {tag}
                      <X
                        className="w-3 h-3 cursor-pointer"
                        onClick={() => removeTag(tag)}
                      />
                    </Badge>
                  ))}
                </div>
                <div className="flex gap-2">
                  <Input
                    value={newTag}
                    onChange={(e) => setNewTag(e.target.value)}
                    placeholder="Add a tag"
                    onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addTag())}
                  />
                  <Button type="button" onClick={addTag} variant="outline" size="sm">
                    <Plus className="w-4 h-4" />
                  </Button>
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium mb-2">Definition</label>
                <Textarea
                  value={editDefinition}
                  onChange={(e) => setEditDefinition(e.target.value)}
                  rows={8}
                  className="font-mono text-sm"
                />
              </div>

              <div className="flex justify-end space-x-2">
                <Button variant="outline" onClick={() => setEditingAgent(null)}>
                  Cancel
                </Button>
                <Button onClick={handleUpdate}>
                  Update Agent
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      </div>
    </div>
  );
}