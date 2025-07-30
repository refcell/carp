import { useState, useEffect, createContext, useContext, useCallback } from 'react';
import { User, Session } from '@supabase/supabase-js';
import { supabase } from '@/integrations/supabase/client';

interface AuthContextType {
  user: User | null;
  session: Session | null;
  signInWithGitHub: () => Promise<void>;
  signOut: () => Promise<void>;
  loading: boolean;
  refreshTokenIfNeeded: () => Promise<boolean>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [session, setSession] = useState<Session | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    console.log('[Auth] Setting up authentication state listener');
    
    // Set up auth state listener
    const { data: { subscription } } = supabase.auth.onAuthStateChange(
      (event, session) => {
        console.log('[Auth] State change event:', event, {
          hasSession: !!session,
          hasUser: !!session?.user,
          userEmail: session?.user?.email
        });
        
        setSession(session);
        setUser(session?.user ?? null);
        setLoading(false);
      }
    );

    // Check for existing session
    console.log('[Auth] Checking for existing session');
    supabase.auth.getSession().then(({ data: { session } }) => {
      console.log('[Auth] Initial session check:', {
        hasSession: !!session,
        hasUser: !!session?.user,
        userEmail: session?.user?.email
      });
      
      setSession(session);
      setUser(session?.user ?? null);
      setLoading(false);
    });

    return () => {
      console.log('[Auth] Cleaning up auth state listener');
      subscription.unsubscribe();
    };
  }, []);

  const signInWithGitHub = async () => {
    const redirectUrl = `${window.location.origin}/`;
    await supabase.auth.signInWithOAuth({
      provider: 'github',
      options: {
        redirectTo: redirectUrl
      }
    });
  };

  const signOut = async () => {
    console.log('[Auth] User signing out');
    await supabase.auth.signOut();
  };

  // Token refresh mechanism to handle JWT expiration
  const refreshTokenIfNeeded = useCallback(async (): Promise<boolean> => {
    if (!session?.access_token) {
      console.warn('[Auth] No access token available for refresh check');
      return false;
    }
    
    try {
      // Decode JWT to check expiration (basic client-side validation)
      const tokenParts = session.access_token.split('.');
      if (tokenParts.length !== 3) {
        console.error('[Auth] Invalid JWT token format');
        return false;
      }
      
      const payload = JSON.parse(atob(tokenParts[1]));
      const currentTime = Date.now();
      const expirationTime = payload.exp * 1000;
      const timeUntilExpiry = expirationTime - currentTime;
      
      console.log(`[Auth] Token expires in ${Math.round(timeUntilExpiry / 1000 / 60)} minutes`);
      
      // Refresh if token expires within 5 minutes (300000ms)
      const isExpiringSoon = timeUntilExpiry < 5 * 60 * 1000;
      
      if (isExpiringSoon) {
        console.log('[Auth] Token expiring soon, attempting refresh');
        const { data, error } = await supabase.auth.refreshSession();
        
        if (error) {
          console.error('[Auth] Token refresh failed:', error.message);
          return false;
        }
        
        if (data.session) {
          console.log('[Auth] Token refresh successful');
          // Session state will be updated by the auth state change listener
          return true;
        } else {
          console.error('[Auth] Token refresh returned no session');
          return false;
        }
      }
      
      // Token is still valid
      return true;
    } catch (error) {
      console.error('[Auth] Token validation error:', error);
      return false;
    }
  }, [session]);

  return (
    <AuthContext.Provider value={{
      user,
      session,
      signInWithGitHub,
      signOut,
      loading,
      refreshTokenIfNeeded
    }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}