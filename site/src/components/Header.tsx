import { useAuth } from '@/hooks/useAuth';
import { Button } from '@/components/ui/button';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Github, LogOut, User, Plus, BookOpen, Upload, Grid3X3, Home } from 'lucide-react';
import { Link, useNavigate } from 'react-router-dom';
import { useRef } from 'react';
import { useToast } from '@/hooks/use-toast';

export function Header() {
  const { user, signInWithGitHub, signOut } = useAuth();
  const navigate = useNavigate();
  const { toast } = useToast();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const parseAgentFile = (content: string) => {
    // Parse name from "name: ..." pattern (case insensitive, handles various whitespace)
    const nameMatch = content.match(/^name\s*:\s*(.+)$/mi);
    const name = nameMatch ? nameMatch[1].trim() : '';

    // Parse description from "description: ..." pattern (case insensitive, handles various whitespace)
    const descriptionMatch = content.match(/^description\s*:\s*(.+)$/mi);
    const description = descriptionMatch ? descriptionMatch[1].trim() : '';

    return { name, description, content };
  };

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    // Validate file type
    if (!file.name.endsWith('.md')) {
      toast({
        title: "Invalid file type",
        description: "Please select a markdown file (.md)",
        variant: "destructive"
      });
      return;
    }

    try {
      const content = await file.text();
      const { name, description } = parseAgentFile(content);

      // Show success message
      toast({
        title: "File uploaded successfully",
        description: `Loaded agent${name ? ` "${name}"` : ''} - review and customize before publishing`,
      });

      // Navigate to create page with pre-populated data
      const searchParams = new URLSearchParams({
        upload: 'true',
        name: name || '',
        description: description || '',
        content: content
      });
      
      navigate(`/create?${searchParams.toString()}`);
    } catch (error) {
      console.error('Error reading file:', error);
      toast({
        title: "Error reading file",
        description: "Failed to read the uploaded file. Please ensure it's a valid markdown file.",
        variant: "destructive"
      });
    } finally {
      // Reset file input
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    }
  };

  const handleUploadClick = () => {
    if (!user) {
      toast({
        title: "Sign in required",
        description: "Please sign in to upload an agent",
        variant: "destructive"
      });
      return;
    }
    
    // Show informative toast about file selection
    toast({
      title: "Select Agent File",
      description: "Please select a markdown (.md) file containing your agent definition",
    });
    
    fileInputRef.current?.click();
  };

  return (
    <header className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="container mx-auto px-4 h-16 flex items-center justify-between">
        <div className="flex items-center space-x-6">
          <Link to="/" className="flex items-center">
            <span className="font-semibold text-lg">Claude Agents</span>
          </Link>
        </div>

        {user ? (
          <div className="flex items-center space-x-4">
            <Button variant="ghost" size="sm" asChild>
              <Link to="/">
                <Home className="w-4 h-4 mr-2" />
                Home
              </Link>
            </Button>
            <Button variant="ghost" size="sm" asChild>
              <Link to="/all-agents">
                <Grid3X3 className="w-4 h-4 mr-2" />
                All Agents
              </Link>
            </Button>
            <Button variant="ghost" size="sm" asChild>
              <Link to="/usage">
                <BookOpen className="w-4 h-4 mr-2" />
                Usage
              </Link>
            </Button>
            <Button variant="outline" size="sm" asChild>
              <Link to="/create">
                <Plus className="w-4 h-4 mr-2" />
                Create Agent
              </Link>
            </Button>
            <Button variant="outline" size="sm" onClick={handleUploadClick}>
              <Upload className="w-4 h-4 mr-2" />
              Upload Agent
            </Button>
            <input
              ref={fileInputRef}
              type="file"
              accept=".md"
              onChange={handleFileUpload}
              className="hidden"
            />
            
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="sm" className="flex items-center space-x-2">
                  <Avatar className="w-6 h-6">
                    <AvatarImage src={user.user_metadata?.avatar_url} />
                    <AvatarFallback>
                      {user.user_metadata?.user_name?.[0]?.toUpperCase() || 'U'}
                    </AvatarFallback>
                  </Avatar>
                  <span className="hidden sm:inline">{user.user_metadata?.user_name}</span>
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem asChild>
                  <Link to="/profile">
                    <User className="w-4 h-4 mr-2" />
                    Profile
                  </Link>
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem onClick={signOut}>
                  <LogOut className="w-4 h-4 mr-2" />
                  Sign Out
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        ) : (
          <div className="flex items-center space-x-4">
            <Button variant="ghost" size="sm" asChild>
              <Link to="/">
                <Home className="w-4 h-4 mr-2" />
                Home
              </Link>
            </Button>
            <Button variant="ghost" size="sm" asChild>
              <Link to="/all-agents">
                <Grid3X3 className="w-4 h-4 mr-2" />
                All Agents
              </Link>
            </Button>
            <Button variant="ghost" size="sm" asChild>
              <Link to="/usage">
                <BookOpen className="w-4 h-4 mr-2" />
                Usage
              </Link>
            </Button>
            <Button onClick={signInWithGitHub} size="sm">
              <Github className="w-4 h-4 mr-2" />
              Sign in with GitHub
            </Button>
          </div>
        )}
      </div>
    </header>
  );
}