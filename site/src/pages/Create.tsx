import { useState, useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useAuth } from '@/hooks/useAuth';
import { useAgents } from '@/hooks/useAgents';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { X, Plus } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';

export default function Create() {
  const { user } = useAuth();
  const { createAgent } = useAgents();
  const navigate = useNavigate();
  const { toast } = useToast();
  const [searchParams] = useSearchParams();
  
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [definition, setDefinition] = useState('');
  const [tags, setTags] = useState<string[]>([]);
  const [newTag, setNewTag] = useState('');
  const [loading, setLoading] = useState(false);
  const [isUploadMode, setIsUploadMode] = useState(false);

  // Handle URL parameters for uploaded agent data
  useEffect(() => {
    const uploadFlag = searchParams.get('upload');
    const uploadedName = searchParams.get('name');
    const uploadedDescription = searchParams.get('description');
    const uploadedContent = searchParams.get('content');

    if (uploadFlag === 'true') {
      setIsUploadMode(true);
      if (uploadedName) setName(uploadedName);
      if (uploadedDescription) setDescription(uploadedDescription);
      if (uploadedContent) setDefinition(uploadedContent);

      // Clear URL parameters after loading data
      navigate('/create', { replace: true });
    }
  }, [searchParams, navigate]);

  if (!user) {
    return (
      <div className="container mx-auto px-4 py-8">
        <Card className="max-w-md mx-auto text-center">
          <CardContent className="pt-6">
            <p className="text-muted-foreground">Please sign in to create an agent</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  const addTag = () => {
    if (newTag.trim() && !tags.includes(newTag.trim())) {
      setTags([...tags, newTag.trim()]);
      setNewTag('');
    }
  };

  const removeTag = (tagToRemove: string) => {
    setTags(tags.filter(tag => tag !== tagToRemove));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name || !description || !definition) return;

    setLoading(true);
    try {
      let parsedDefinition;
      try {
        parsedDefinition = JSON.parse(definition);
      } catch {
        parsedDefinition = { prompt: definition };
      }

      await createAgent({
        name,
        description,
        definition: parsedDefinition,
        tags: tags.length > 0 ? tags : null
      });

      toast({
        title: "Agent created!",
        description: "Your agent has been successfully created."
      });

      navigate('/profile');
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to create agent. Please try again.",
        variant: "destructive"
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-2xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold">
            {isUploadMode ? 'Upload Agent' : 'Create New Agent'}
          </h1>
          <p className="text-muted-foreground mt-2">
            {isUploadMode 
              ? 'Review and customize your uploaded agent before sharing'
              : 'Share your Claude agent with the community'
            }
          </p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>Agent Details</CardTitle>
            <CardDescription>
              {isUploadMode 
                ? 'Review and modify the details parsed from your uploaded agent file'
                : 'Provide information about your Claude agent'
              }
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-6">
              <div>
                <label htmlFor="name" className="block text-sm font-medium mb-2">
                  Name
                </label>
                <Input
                  id="name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g., Code Reviewer, Writing Assistant"
                  required
                />
              </div>

              <div>
                <label htmlFor="description" className="block text-sm font-medium mb-2">
                  Description
                </label>
                <Textarea
                  id="description"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Describe what your agent does and how it helps users..."
                  rows={3}
                  required
                />
              </div>

              <div>
                <label htmlFor="tags" className="block text-sm font-medium mb-2">
                  Tags
                </label>
                <div className="flex flex-wrap gap-2 mb-2">
                  {tags.map((tag) => (
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
                <label htmlFor="definition" className="block text-sm font-medium mb-2">
                  Agent Definition {isUploadMode && '(from uploaded file)'}
                </label>
                <Textarea
                  id="definition"
                  value={definition}
                  onChange={(e) => setDefinition(e.target.value)}
                  placeholder="Enter your agent's prompt or JSON definition..."
                  rows={8}
                  required
                  className="font-mono text-sm"
                />
                <p className="text-xs text-muted-foreground mt-1">
                  You can paste a JSON object or just the prompt text
                </p>
              </div>

              <div className="flex gap-4">
                <Button type="submit" disabled={loading} className="flex-1">
                  {loading ? 'Creating...' : 'Create Agent'}
                </Button>
                <Button type="button" variant="outline" onClick={() => navigate(-1)}>
                  Cancel
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}