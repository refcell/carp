import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AuthProvider } from "@/hooks/useAuth";
import { ThemeProvider } from "@/hooks/useTheme";
import { Header } from "@/components/Header";
import { ThemeToggle } from "@/components/ThemeToggle";
import Index from "./pages/Index";
import Create from "./pages/Create";
import Profile from "./pages/Profile";
import Usage from "./pages/Usage";
import AllAgents from "./pages/AllAgents";
import NotFound from "./pages/NotFound";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      gcTime: 10 * 60 * 1000, // 10 minutes
      refetchOnWindowFocus: false,
      refetchOnReconnect: 'always',
      retry: (failureCount, error: unknown) => {
        // Don't retry on 4xx errors
        if (error?.status >= 400 && error?.status < 500) {
          return false;
        }
        return failureCount < 3;
      },
    },
    mutations: {
      retry: 1,
    },
  },
});

const App = () => (
  <QueryClientProvider client={queryClient}>
    <ThemeProvider defaultTheme="dark">
      <AuthProvider>
        <TooltipProvider>
          <Toaster />
          <Sonner />
          <BrowserRouter>
            <div className="min-h-screen flex flex-col bg-background">
              <Header />
              <Routes>
                <Route path="/" element={<Index />} />
                <Route path="/create" element={<Create />} />
                <Route path="/profile" element={<Profile />} />
                <Route path="/usage" element={<Usage />} />
                <Route path="/all-agents" element={<AllAgents />} />
                <Route path="*" element={<NotFound />} />
              </Routes>
              <ThemeToggle />
            </div>
          </BrowserRouter>
        </TooltipProvider>
      </AuthProvider>
    </ThemeProvider>
  </QueryClientProvider>
);

export default App;
