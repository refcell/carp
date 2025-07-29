import { useState, useEffect } from 'react';
import { useApiKeys, ApiKey, ApiKeyWithSecret } from '@/hooks/useApiKeys';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { useToast } from '@/hooks/use-toast';
import { Key, Copy, Trash2, Plus, Eye, EyeOff, Calendar } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

interface ApiKeyManagementModalProps {
  children: React.ReactNode;
}

export function ApiKeyManagementModal({ children }: ApiKeyManagementModalProps) {
  const { loading, fetchApiKeys, createApiKey, deleteApiKey } = useApiKeys();
  const { toast } = useToast();
  
  const [isOpen, setIsOpen] = useState(false);
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [newKeyName, setNewKeyName] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [newlyCreatedKey, setNewlyCreatedKey] = useState<ApiKeyWithSecret | null>(null);
  const [showNewKey, setShowNewKey] = useState(false);

  // Load API keys when modal opens
  useEffect(() => {
    if (isOpen) {
      loadApiKeys();
    }
  }, [isOpen]);

  const loadApiKeys = async () => {
    const keys = await fetchApiKeys();
    setApiKeys(keys);
  };

  const handleCreateKey = async () => {
    if (!newKeyName.trim()) {
      toast({
        title: "Name required",
        description: "Please enter a name for your API key.",
        variant: "destructive"
      });
      return;
    }

    setIsCreating(true);
    try {
      const newKey = await createApiKey(newKeyName.trim());
      if (newKey) {
        setNewlyCreatedKey(newKey);
        setShowNewKey(true);
        setNewKeyName('');
        await loadApiKeys();
        
        toast({
          title: "API Key Created",
          description: "Your new API key has been created successfully. Make sure to copy it now!"
        });
      }
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to create API key. Please try again.",
        variant: "destructive"
      });
    } finally {
      setIsCreating(false);
    }
  };

  const handleDeleteKey = async (keyId: string, keyName: string) => {
    if (!confirm(`Are you sure you want to delete "${keyName}"? This action cannot be undone.`)) {
      return;
    }

    try {
      await deleteApiKey(keyId);
      await loadApiKeys();
      
      toast({
        title: "API Key Deleted",
        description: "The API key has been successfully deleted."
      });
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to delete API key. Please try again.",
        variant: "destructive"
      });
    }
  };

  const copyToClipboard = async (text: string, description: string) => {
    try {
      await navigator.clipboard.writeText(text);
      toast({
        title: "Copied!",
        description: `${description} copied to clipboard`
      });
    } catch (error) {
      toast({
        title: "Copy failed",
        description: "Failed to copy to clipboard. Please copy manually.",
        variant: "destructive"
      });
    }
  };

  const handleModalOpenChange = (open: boolean) => {
    setIsOpen(open);
    if (!open) {
      // Clear newly created key when modal closes
      setNewlyCreatedKey(null);
      setShowNewKey(false);
      setNewKeyName('');
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleModalOpenChange}>
      <DialogTrigger asChild>
        {children}
      </DialogTrigger>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Key className="w-5 h-5" />
            API Key Management
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-6">
          {/* Overview */}
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-lg font-medium">Your API Keys</h3>
              <p className="text-sm text-muted-foreground">
                {apiKeys.length} API key{apiKeys.length !== 1 ? 's' : ''} configured
              </p>
            </div>
            <Badge variant="secondary" className="text-sm">
              {apiKeys.length} / 10 keys
            </Badge>
          </div>

          {/* Newly Created Key Display */}
          {newlyCreatedKey && (
            <Card className="border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950">
              <CardHeader>
                <CardTitle className="text-green-800 dark:text-green-200 flex items-center gap-2">
                  <Key className="w-4 h-4" />
                  New API Key Created
                </CardTitle>
                <CardDescription className="text-green-700 dark:text-green-300">
                  This is the only time you'll see the full key. Make sure to copy it now!
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div>
                  <label className="text-xs font-medium text-green-800 dark:text-green-200 uppercase tracking-wide">
                    Key Name
                  </label>
                  <p className="text-sm font-mono mt-1">{newlyCreatedKey.name}</p>
                </div>
                <div>
                  <label className="text-xs font-medium text-green-800 dark:text-green-200 uppercase tracking-wide">
                    API Key
                  </label>
                  <div className="flex items-center gap-2 mt-1">
                    <Input
                      type={showNewKey ? "text" : "password"}
                      value={newlyCreatedKey.full_key}
                      readOnly
                      className="font-mono text-sm"
                    />
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => setShowNewKey(!showNewKey)}
                      className="shrink-0"
                    >
                      {showNewKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                    </Button>
                    <Button
                      size="sm"
                      onClick={() => copyToClipboard(newlyCreatedKey.full_key, "API key")}
                      className="shrink-0"
                    >
                      <Copy className="w-4 h-4" />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Create New Key */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Create New API Key</CardTitle>
              <CardDescription>
                Generate a new API key for accessing the Carp API
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <label className="text-sm font-medium">Key Name</label>
                <Input
                  value={newKeyName}
                  onChange={(e) => setNewKeyName(e.target.value)}
                  placeholder="e.g., Production Server, Development"
                  className="mt-1"
                />
              </div>
              <Button 
                onClick={handleCreateKey}
                disabled={isCreating || !newKeyName.trim()}
                className="w-full"
              >
                {isCreating ? (
                  "Creating..."
                ) : (
                  <>
                    <Plus className="w-4 h-4 mr-2" />
                    Create API Key
                  </>
                )}
              </Button>
            </CardContent>
          </Card>

          <Separator />

          {/* Existing Keys */}
          <div>
            <h3 className="text-base font-medium mb-4">Existing API Keys</h3>
            
            {loading ? (
              <div className="text-center py-8">
                <p className="text-muted-foreground">Loading API keys...</p>
              </div>
            ) : apiKeys.length === 0 ? (
              <Card>
                <CardContent className="text-center py-8">
                  <Key className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
                  <p className="text-muted-foreground mb-2">No API keys created yet</p>
                  <p className="text-sm text-muted-foreground">
                    Create your first API key to start using the Carp API
                  </p>
                </CardContent>
              </Card>
            ) : (
              <div className="space-y-3">
                {apiKeys.map((key) => (
                  <Card key={key.id}>
                    <CardContent className="pt-4">
                      <div className="flex items-center justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-2">
                            <h4 className="font-medium">
                              {key.name}
                            </h4>
                            <Badge variant="outline" className="text-xs">
                              {key.prefix}...
                            </Badge>
                          </div>
                          <div className="flex items-center gap-4 text-sm text-muted-foreground">
                            <div className="flex items-center gap-1">
                              <Calendar className="w-3 h-3" />
                              Created {formatDistanceToNow(new Date(key.created_at), { addSuffix: true })}
                            </div>
                            {key.last_used_at && (
                              <div className="text-xs">
                                Last used {formatDistanceToNow(new Date(key.last_used_at), { addSuffix: true })}
                              </div>
                            )}
                            <Badge variant={key.is_active ? "secondary" : "outline"} className="text-xs">
                              {key.is_active ? "Active" : "Inactive"}
                            </Badge>
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => copyToClipboard(key.prefix, "Key prefix")}
                          >
                            <Copy className="w-3 h-3" />
                          </Button>
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => handleDeleteKey(key.id, key.name)}
                            className="text-destructive hover:text-destructive"
                          >
                            <Trash2 className="w-3 h-3" />
                          </Button>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </div>

          {/* Usage Information */}
          <Card className="bg-muted/50">
            <CardHeader>
              <CardTitle className="text-base">Usage Information</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              <p>• API keys are used to authenticate requests to the Carp API</p>
              <p>• Include your API key in the <code className="bg-muted px-1 py-0.5 rounded text-xs">Authorization</code> header</p>
              <p>• Keys are only shown in full when first created - store them securely</p>
              <p>• You can create up to 10 API keys per account</p>
            </CardContent>
          </Card>
        </div>
      </DialogContent>
    </Dialog>
  );
}