import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import {
  Clock,
  Download,
  Search,
  Star,
  Copy,
  FileText,
  Users,
  Zap,
  CheckCircle,
  AlertCircle,
  BookOpen,
  MessageSquare,
  GitFork,
  TrendingUp,
  Code,
  FileDown,
  Target,
  Lightbulb
} from 'lucide-react';
import { useEffect } from 'react';
import { Link } from 'react-router-dom';

const Usage = () => {
  useEffect(() => {
    document.title = 'Usage Guide - Claude Agent Registry';
  }, []);

  const scrollToSection = (sectionId: string) => {
    const element = document.getElementById(sectionId);
    if (element) {
      element.scrollIntoView({ behavior: 'smooth' });
    }
  };

  return (
    <div className="min-h-screen bg-background">
      {/* Hero Section */}
      <section className="pt-16 pb-4">
        <div className="container mx-auto text-center">
          <div className="max-w-4xl mx-auto">
            <h1 className="text-4xl pb-2 md:text-6xl font-bold mb-6 bg-gradient-to-r from-primary to-accent bg-clip-text text-transparent">
              Usage Guide
            </h1>
            <p className="text-xl text-muted-foreground mb-8 max-w-2xl mx-auto">
              Learn how to discover, install, and use Claude agents from our community registry.
              Transform your workflow from manual setup to one-click installation.
            </p>
          </div>
        </div>
      </section>

      {/* TL;DR Section */}
      <section className="container mx-auto pb-2">
        <div className="max-w-4xl mx-auto">
          <Card className="bg-gradient-to-r from-amber-50 to-yellow-50 dark:from-amber-900/20 dark:to-yellow-900/20 border border-amber-200 dark:border-amber-700/50 mb-8">
            <CardHeader>
              <CardTitle className="flex items-center space-x-2 text-lg">
                <Lightbulb className="w-5 h-5 text-amber-500" />
                <span>TL;DR - Quick Start</span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex items-start space-x-3">
                <Badge className="mt-1 bg-amber-500 hover:bg-amber-600">Quick</Badge>
                <div>
                  <p className="text-sm font-medium mb-1">
                    Copy an agent's configuration to your <code className="bg-muted px-1.5 py-0.5 rounded text-xs font-mono">~/.claude/agents/</code> directory and restart Claude.
                  </p>
                  <p className="text-xs text-muted-foreground">
                    Browse agents below → Click "Copy Configuration" → Paste into <code className="bg-muted px-1 py-0.5 rounded font-mono">~/.claude/agents/agent-name/config.json</code> → Restart Claude → Use with <code className="bg-muted px-1 py-0.5 rounded font-mono">@agent-name</code>
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </section>

      {/* Table of Contents */}
      <section className="container mx-auto pb-2">
        <div className="max-w-4xl mx-auto">
          <Card className="mb-8">
            <CardHeader>
              <CardTitle className="flex items-center space-x-2">
                <BookOpen className="w-5 h-5" />
                <span>Table of Contents</span>
              </CardTitle>
              <CardDescription>Jump to any section quickly</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-3">
                  <Button
                    variant="ghost"
                    className="justify-start h-auto py-3 px-4 text-left w-full hover:bg-muted/50 transition-colors"
                    onClick={() => scrollToSection('problem')}
                  >
                    <AlertCircle className="w-4 h-4 mr-3 text-red-500 flex-shrink-0" />
                    <span className="text-sm font-medium">The Problem: Manual Setup</span>
                  </Button>
                  <Button
                    variant="ghost"
                    className="justify-start h-auto py-3 px-4 text-left w-full hover:bg-muted/50 transition-colors"
                    onClick={() => scrollToSection('solution')}
                  >
                    <Zap className="w-4 h-4 mr-3 text-emerald-500 flex-shrink-0" />
                    <span className="text-sm font-medium">The Solution: Agent Registry</span>
                  </Button>
                  <Button
                    variant="ghost"
                    className="justify-start h-auto py-3 px-4 text-left w-full hover:bg-muted/50 transition-colors"
                    onClick={() => scrollToSection('discovery')}
                  >
                    <Search className="w-4 h-4 mr-3 text-blue-500 flex-shrink-0" />
                    <span className="text-sm font-medium">Discovering Agents</span>
                  </Button>
                </div>
                <div className="space-y-3">
                  <Button
                    variant="ghost"
                    className="justify-start h-auto py-3 px-4 text-left w-full hover:bg-muted/50 transition-colors"
                    onClick={() => scrollToSection('installation')}
                  >
                    <Download className="w-4 h-4 mr-3 text-purple-500 flex-shrink-0" />
                    <span className="text-sm font-medium">Installing Agents</span>
                  </Button>
                  <Button
                    variant="ghost"
                    className="justify-start h-auto py-3 px-4 text-left w-full hover:bg-muted/50 transition-colors"
                    onClick={() => scrollToSection('usage-tips')}
                  >
                    <Target className="w-4 h-4 mr-3 text-orange-500 flex-shrink-0" />
                    <span className="text-sm font-medium">Usage Tips</span>
                  </Button>
                  <Button
                    variant="ghost"
                    className="justify-start h-auto py-3 px-4 text-left w-full hover:bg-muted/50 transition-colors"
                    onClick={() => scrollToSection('support')}
                  >
                    <MessageSquare className="w-4 h-4 mr-3 text-indigo-500 flex-shrink-0" />
                    <span className="text-sm font-medium">Getting Help</span>
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </section>

      {/* Main Content */}
      <section className="container mx-auto pb-16">
        <div className="max-w-4xl mx-auto space-y-12">

          {/* The Problem Section */}
          <div id="problem">
            <Card className="border-red-500/30 bg-red-950/50 dark:bg-red-900/20">
              <CardHeader>
                <CardTitle className="flex items-center space-x-2 text-2xl">
                  <AlertCircle className="w-6 h-6 text-destructive" />
                  <span>The Problem: Manual Agent Configuration is Tedious</span>
                </CardTitle>
                <CardDescription>
                  Currently, Claude users face significant friction when setting up custom agents.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div>
                  <h3 className="text-lg font-semibold mb-4 flex items-center space-x-2">
                    <Clock className="w-5 h-5" />
                    <span>The Manual Process</span>
                  </h3>
                  <div className="space-y-3">
                    <div className="flex items-start space-x-3">
                      <Badge variant="secondary" className="mt-1">1</Badge>
                      <p className="text-muted-foreground">Create agent directories manually in <code className="bg-muted px-1 py-0.5 rounded text-sm">~/.claude/agents/</code></p>
                    </div>
                    <div className="flex items-start space-x-3">
                      <Badge variant="secondary" className="mt-1">2</Badge>
                      <p className="text-muted-foreground">Write complex JSON or YAML configuration files from scratch</p>
                    </div>
                    <div className="flex items-start space-x-3">
                      <Badge variant="secondary" className="mt-1">3</Badge>
                      <p className="text-muted-foreground">Define system prompts, capabilities, and parameters without guidance</p>
                    </div>
                    <div className="flex items-start space-x-3">
                      <Badge variant="secondary" className="mt-1">4</Badge>
                      <p className="text-muted-foreground">Test and debug configurations through trial and error</p>
                    </div>
                    <div className="flex items-start space-x-3">
                      <Badge variant="secondary" className="mt-1">5</Badge>
                      <p className="text-muted-foreground">Repeat this process for every new agent you want to use</p>
                    </div>
                  </div>
                </div>

                <Separator />

                <div>
                  <h3 className="text-lg font-semibold mb-4">Why This Creates Problems</h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="space-y-4">
                      <div>
                        <h4 className="font-medium text-destructive mb-2">Time-Consuming Setup</h4>
                        <p className="text-sm text-muted-foreground">
                          Setting up even a simple agent can take 30-60 minutes. Complex agents with multiple capabilities can extend to several hours.
                        </p>
                      </div>
                      <div>
                        <h4 className="font-medium text-destructive mb-2">Steep Learning Curve</h4>
                        <p className="text-sm text-muted-foreground">
                          New users must understand configuration syntax, prompt engineering, parameter tuning, and file structure conventions.
                        </p>
                      </div>
                      <div>
                        <h4 className="font-medium text-destructive mb-2">No Discoverability</h4>
                        <p className="text-sm text-muted-foreground">
                          No central place to discover existing agents, leading to wasted effort and inconsistent quality.
                        </p>
                      </div>
                    </div>
                    <div className="space-y-4">
                      <div>
                        <h4 className="font-medium text-destructive mb-2">Duplication Across Users</h4>
                        <p className="text-sm text-muted-foreground">
                          Thousands of users independently create similar agents for code review, writing, data analysis, and customer support.
                        </p>
                      </div>
                      <div>
                        <h4 className="font-medium text-destructive mb-2">Maintenance Overhead</h4>
                        <p className="text-sm text-muted-foreground">
                          Keeping configurations updated, sharing with teams, version control, and troubleshooting issues.
                        </p>
                      </div>
                      <div>
                        <h4 className="font-medium text-destructive mb-2">Lack of Quality Standards</h4>
                        <p className="text-sm text-muted-foreground">
                          Without shared examples, agent quality varies widely with poor prompts and security vulnerabilities.
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* The Solution Section */}
          <div id="solution">
            <Card className="border-emerald-500/30 bg-emerald-950/50 dark:bg-emerald-900/20">
              <CardHeader>
                <CardTitle className="flex items-center space-x-2 text-2xl">
                  <Zap className="w-6 h-6 text-primary" />
                  <span>The Solution: A Central Agent Registry</span>
                </CardTitle>
                <CardDescription>
                  Transform agent management from a manual chore into a streamlined experience.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center space-x-2 text-lg">
                        <CheckCircle className="w-5 h-5 text-green-500" />
                        <span>Instant Access to Quality Agents</span>
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <div>
                        <h4 className="font-medium mb-1">Pre-Built and Tested</h4>
                        <p className="text-sm text-muted-foreground">
                          Every agent has been crafted and tested by experienced prompt engineers and the community.
                        </p>
                      </div>
                      <div>
                        <h4 className="font-medium mb-1">One-Click Installation</h4>
                        <p className="text-sm text-muted-foreground">
                          Simply browse, click, and copy. No more writing configuration files from scratch.
                        </p>
                      </div>
                      <div>
                        <h4 className="font-medium mb-1">Comprehensive Documentation</h4>
                        <p className="text-sm text-muted-foreground">
                          Clear usage instructions, examples, configuration explanations, and performance tips.
                        </p>
                      </div>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center space-x-2 text-lg">
                        <Users className="w-5 h-5 text-blue-500" />
                        <span>Community-Powered Discovery</span>
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-3">
                      <div>
                        <h4 className="font-medium mb-1">Curated Collections</h4>
                        <p className="text-sm text-muted-foreground">
                          Organized by category: Development, Writing, Analysis, Business, and Education.
                        </p>
                      </div>
                      <div>
                        <h4 className="font-medium mb-1">User Ratings and Reviews</h4>
                        <p className="text-sm text-muted-foreground">
                          Community feedback on performance, strengths, limitations, and use cases.
                        </p>
                      </div>
                      <div>
                        <h4 className="font-medium mb-1">Version History</h4>
                        <p className="text-sm text-muted-foreground">
                          Track improvements over time and choose the version that works best.
                        </p>
                      </div>
                    </CardContent>
                  </Card>
                </div>

                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center space-x-2 text-lg">
                      <GitFork className="w-5 h-5 text-accent" />
                      <span>Collaboration and Sharing</span>
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                      <div>
                        <h4 className="font-medium mb-2">Community Contributions</h4>
                        <p className="text-sm text-muted-foreground">
                          Expert prompt engineers share their best work, creating a growing library.
                        </p>
                      </div>
                      <div>
                        <h4 className="font-medium mb-2">Fork and Customize</h4>
                        <p className="text-sm text-muted-foreground">
                          Start with proven configurations and modify for your specific needs.
                        </p>
                      </div>
                      <div>
                        <h4 className="font-medium mb-2">Team Collaboration</h4>
                        <p className="text-sm text-muted-foreground">
                          Share configurations across your organization with consistent quality.
                        </p>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </CardContent>
            </Card>
          </div>

          {/* Discovering Agents Section */}
          <div id="discovery">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2 text-2xl">
                  <Search className="w-6 h-6" />
                  <span>Discovering Agents</span>
                </CardTitle>
                <CardDescription>
                  Multiple ways to find the perfect agent for your needs
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center space-x-2 text-lg">
                        <FileText className="w-5 h-5" />
                        <span>Browse by Category</span>
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <ol className="text-sm text-muted-foreground space-y-1">
                        <li>1. Navigate to the main directory</li>
                        <li>2. Click your area of interest</li>
                        <li>3. Browse agents with descriptions</li>
                      </ol>
                      <div className="mt-3 flex flex-wrap gap-1">
                        <Badge variant="outline">Development</Badge>
                        <Badge variant="outline">Writing</Badge>
                        <Badge variant="outline">Analysis</Badge>
                        <Badge variant="outline">Business</Badge>
                        <Badge variant="outline">Education</Badge>
                      </div>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center space-x-2 text-lg">
                        <Search className="w-5 h-5" />
                        <span>Search Functionality</span>
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="text-sm text-muted-foreground space-y-2">
                        <p>Search by keywords like:</p>
                        <ul className="list-disc list-inside space-y-1">
                          <li>"code review"</li>
                          <li>"data analysis"</li>
                          <li>"customer support"</li>
                        </ul>
                        <p>Filter by ratings, popularity, or recent updates</p>
                      </div>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center space-x-2 text-lg">
                        <TrendingUp className="w-5 h-5" />
                        <span>Featured Agents</span>
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="text-sm text-muted-foreground space-y-2">
                        <p>Check our homepage for:</p>
                        <ul className="list-disc list-inside space-y-1">
                          <li>Most popular agents</li>
                          <li>Recently updated configs</li>
                          <li>Community recommendations</li>
                          <li>New releases</li>
                        </ul>
                      </div>
                    </CardContent>
                  </Card>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Installing Agents Section */}
          <div id="installation">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2 text-2xl">
                  <Download className="w-6 h-6" />
                  <span>Installing Agents</span>
                </CardTitle>
                <CardDescription>
                  Two simple methods to get agents running on your system
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <Card className="border-primary/20">
                    <CardHeader>
                      <CardTitle className="flex items-center space-x-2">
                        <Copy className="w-5 h-5 text-primary" />
                        <span>Quick Copy Method</span>
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-3">
                        <div className="flex items-start space-x-3">
                          <Badge className="mt-1">1</Badge>
                          <div>
                            <p className="font-medium">Find Your Agent</p>
                            <p className="text-sm text-muted-foreground">Browse or search for the agent you want</p>
                          </div>
                        </div>
                        <div className="flex items-start space-x-3">
                          <Badge className="mt-1">2</Badge>
                          <div>
                            <p className="font-medium">Click "Copy Configuration"</p>
                            <p className="text-sm text-muted-foreground">Copies the complete configuration to clipboard</p>
                          </div>
                        </div>
                        <div className="flex items-start space-x-3">
                          <Badge className="mt-1">3</Badge>
                          <div>
                            <p className="font-medium">Create Agent File</p>
                            <p className="text-sm text-muted-foreground">Navigate to <code className="bg-muted px-1 py-0.5 rounded">~/.claude/agents/</code></p>
                          </div>
                        </div>
                        <div className="flex items-start space-x-3">
                          <Badge className="mt-1">4</Badge>
                          <div>
                            <p className="font-medium">Paste Configuration</p>
                            <p className="text-sm text-muted-foreground">Create <code className="bg-muted px-1 py-0.5 rounded">config.json</code> and paste</p>
                          </div>
                        </div>
                      </div>
                    </CardContent>
                  </Card>

                  <Card className="border-accent/20">
                    <CardHeader>
                      <CardTitle className="flex items-center space-x-2">
                        <FileDown className="w-5 h-5 text-accent" />
                        <span>Direct Download</span>
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-3">
                        <p className="text-sm text-muted-foreground mb-3">
                          For agents with multiple files or complex setups:
                        </p>
                        <div className="flex items-start space-x-3">
                          <Badge variant="secondary" className="mt-1">1</Badge>
                          <div>
                            <p className="font-medium">Download Package</p>
                            <p className="text-sm text-muted-foreground">Click "Download Agent Package"</p>
                          </div>
                        </div>
                        <div className="flex items-start space-x-3">
                          <Badge variant="secondary" className="mt-1">2</Badge>
                          <div>
                            <p className="font-medium">Extract Files</p>
                            <p className="text-sm text-muted-foreground">Extract to <code className="bg-muted px-1 py-0.5 rounded">~/.claude/agents/</code></p>
                          </div>
                        </div>
                        <div className="flex items-start space-x-3">
                          <Badge variant="secondary" className="mt-1">3</Badge>
                          <div>
                            <p className="font-medium">Follow Instructions</p>
                            <p className="text-sm text-muted-foreground">Read any included setup instructions</p>
                          </div>
                        </div>
                        <div className="flex items-start space-x-3">
                          <Badge variant="secondary" className="mt-1">4</Badge>
                          <div>
                            <p className="font-medium">Restart Claude</p>
                            <p className="text-sm text-muted-foreground">Restart if necessary</p>
                          </div>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                </div>

                <Card className="bg-slate-100/50 dark:bg-slate-800/30">
                  <CardHeader>
                    <CardTitle className="flex items-center space-x-2">
                      <Code className="w-5 h-5" />
                      <span>Using Your Installed Agents</span>
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-4">
                      <div>
                        <h4 className="font-medium mb-2">Activating an Agent</h4>
                        <p className="text-sm text-muted-foreground mb-2">
                          Once installed, agents appear in your Claude interface:
                        </p>
                        <div className="bg-muted p-3 rounded-md">
                          <code className="text-sm">@agent-name your message here</code>
                        </div>
                        <p className="text-sm text-muted-foreground mt-2">
                          Or select from the agent dropdown menu in supported Claude interfaces.
                        </p>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </CardContent>
            </Card>
          </div>

          {/* Usage Tips Section */}
          <div id="usage-tips">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2 text-2xl">
                  <Target className="w-6 h-6" />
                  <span>Best Practices for Agent Usage</span>
                </CardTitle>
                <CardDescription>
                  Get the most out of your agents with these proven tips
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div className="space-y-4">
                    <div>
                      <h3 className="text-lg font-semibold mb-3 flex items-center space-x-2">
                        <CheckCircle className="w-5 h-5 text-green-500" />
                        <span>Do This</span>
                      </h3>
                      <div className="space-y-3">
                        <div className="p-3 bg-emerald-50 dark:bg-emerald-900/30 border border-emerald-200 dark:border-emerald-700/50 rounded-lg">
                          <p className="text-sm font-medium text-green-700 dark:text-green-400">
                            ✅ "Review this Python function for performance issues and suggest optimizations"
                          </p>
                        </div>
                        <div className="space-y-2">
                          <h4 className="font-medium">Start with Clear Instructions</h4>
                          <p className="text-sm text-muted-foreground">
                            Even well-configured agents perform better with specific, clear requests.
                          </p>
                        </div>
                        <div className="space-y-2">
                          <h4 className="font-medium">Provide Context</h4>
                          <p className="text-sm text-muted-foreground">
                            Share background info, your experience level, and any constraints or requirements.
                          </p>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="space-y-4">
                    <div>
                      <h3 className="text-lg font-semibold mb-3 flex items-center space-x-2">
                        <AlertCircle className="w-5 h-5 text-destructive" />
                        <span>Avoid This</span>
                      </h3>
                      <div className="space-y-3">
                        <div className="p-3 bg-red-50 dark:bg-red-900/30 border border-red-200 dark:border-red-700/50 rounded-lg">
                          <p className="text-sm font-medium text-destructive">
                            ❌ "Look at this code"
                          </p>
                        </div>
                        <div className="space-y-2">
                          <h4 className="font-medium">Understand Agent Capabilities</h4>
                          <p className="text-sm text-muted-foreground">
                            Each agent listing includes strengths, limitations, and best use cases.
                          </p>
                        </div>
                        <div className="space-y-2">
                          <h4 className="font-medium">Iterate and Refine</h4>
                          <p className="text-sm text-muted-foreground">
                            Start with basic requests, provide feedback, and adjust based on the agent's strengths.
                          </p>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <Separator />

                <div>
                  <h3 className="text-lg font-semibold mb-4">Advanced Usage Tips</h3>
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <Card>
                      <CardHeader>
                        <CardTitle className="text-base">Combining Agents</CardTitle>
                      </CardHeader>
                      <CardContent>
                        <p className="text-sm text-muted-foreground mb-2">
                          For complex workflows, use multiple specialized agents:
                        </p>
                        <ul className="text-sm text-muted-foreground space-y-1">
                          <li>• Research agent for information</li>
                          <li>• Writing agent for content</li>
                          <li>• Review agent for quality checks</li>
                        </ul>
                      </CardContent>
                    </Card>

                    <Card>
                      <CardHeader>
                        <CardTitle className="text-base">Customizing Agents</CardTitle>
                      </CardHeader>
                      <CardContent>
                        <p className="text-sm text-muted-foreground mb-2">
                          Most agents can be modified:
                        </p>
                        <ol className="text-sm text-muted-foreground space-y-1">
                          <li>1. Copy base configuration</li>
                          <li>2. Modify prompts/parameters</li>
                          <li>3. Test with your use cases</li>
                          <li>4. Save as custom agent</li>
                        </ol>
                      </CardContent>
                    </Card>

                    <Card>
                      <CardHeader>
                        <CardTitle className="text-base">Sharing Improvements</CardTitle>
                      </CardHeader>
                      <CardContent>
                        <p className="text-sm text-muted-foreground mb-2">
                          Found improvements? Consider:
                        </p>
                        <ul className="text-sm text-muted-foreground space-y-1">
                          <li>• Submit feedback to author</li>
                          <li>• Contribute enhanced version</li>
                          <li>• Create a fork with modifications</li>
                        </ul>
                      </CardContent>
                    </Card>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Getting Help Section */}
          <div id="support">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center space-x-2 text-2xl">
                  <MessageSquare className="w-6 h-6" />
                  <span>Getting Help and Support</span>
                </CardTitle>
                <CardDescription>
                  Resources and community support to help you succeed
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center space-x-2">
                        <BookOpen className="w-5 h-5" />
                        <span>Documentation Resources</span>
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <ul className="space-y-2 text-sm">
                        <li className="flex items-center space-x-2">
                          <Badge variant="outline" className="text-xs">Guide</Badge>
                          <span>Agent Configuration Guide</span>
                        </li>
                        <li className="flex items-center space-x-2">
                          <Badge variant="outline" className="text-xs">Tips</Badge>
                          <span>Prompt Engineering Tips</span>
                        </li>
                        <li className="flex items-center space-x-2">
                          <Badge variant="outline" className="text-xs">Fix</Badge>
                          <span>Troubleshooting Guide</span>
                        </li>
                        <li className="flex items-center space-x-2">
                          <Badge variant="outline" className="text-xs">Best</Badge>
                          <span>Best Practices</span>
                        </li>
                      </ul>
                    </CardContent>
                  </Card>

                  <Card>
                    <CardHeader>
                      <CardTitle className="flex items-center space-x-2">
                        <Users className="w-5 h-5" />
                        <span>Community Support</span>
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <ul className="space-y-2 text-sm">
                        <li className="flex items-center space-x-2">
                          <Badge variant="outline" className="text-xs">Forum</Badge>
                          <span>Discussion Forums</span>
                        </li>
                        <li className="flex items-center space-x-2">
                          <Badge variant="outline" className="text-xs">Request</Badge>
                          <span>Agent Requests</span>
                        </li>
                        <li className="flex items-center space-x-2">
                          <Badge variant="outline" className="text-xs">Bug</Badge>
                          <span>Bug Reports</span>
                        </li>
                        <li className="flex items-center space-x-2">
                          <Badge variant="outline" className="text-xs">Share</Badge>
                          <span>Contribution Guidelines</span>
                        </li>
                      </ul>
                    </CardContent>
                  </Card>
                </div>

                <Card className="bg-gradient-to-br from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 border border-blue-200 dark:border-blue-700/50">
                  <CardHeader>
                    <CardTitle className="flex items-center space-x-2">
                      <TrendingUp className="w-5 h-5 text-primary" />
                      <span>Stay Updated</span>
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <div>
                        <h4 className="font-medium mb-2">Regular Updates</h4>
                        <ul className="text-sm text-muted-foreground space-y-1">
                          <li>• New agent releases</li>
                          <li>• Platform updates</li>
                        </ul>
                      </div>
                      <div>
                        <h4 className="font-medium mb-2">Community Highlights</h4>
                        <ul className="text-sm text-muted-foreground space-y-1">
                          <li>• Success stories</li>
                          <li>• Tips from power users</li>
                        </ul>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </CardContent>
            </Card>
          </div>

          {/* Call to Action */}
          <Card className="bg-gradient-to-r from-purple-50 to-blue-50 dark:from-purple-900/20 dark:to-blue-900/20 border border-purple-200 dark:border-purple-700/50">
            <CardContent className="p-4 text-center">
              <h2 className="text-2xl font-bold mb-4">Ready to Transform Your Claude Experience?</h2>
              <p className="text-lg text-muted-foreground mb-6 max-w-2xl mx-auto">
                Start browsing our agent collection and discover how much time and effort you can save
                with pre-built, community-tested configurations.
              </p>
              <Button size="lg" className="bg-gradient-to-r from-primary to-accent hover:from-primary/90 hover:to-accent/90" asChild>
                <Link to="/">Browse Agents Now</Link>
              </Button>
            </CardContent>
          </Card>
        </div>
      </section>
    </div>
  );
};

export default Usage;
