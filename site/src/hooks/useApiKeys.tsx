import { useState, useCallback } from 'react';
import { useAuth } from './useAuth';
import { getApiBaseUrl, API_ENDPOINTS, createAuthenticatedFetch, ApiRequestError } from '@/lib/api-config';

export interface ApiKey {
  id: string;
  name: string;
  prefix: string;
  scopes: string[];
  is_active: boolean;
  last_used_at: string | null;
  expires_at: string | null;
  created_at: string;
}

export interface ApiKeyWithSecret extends ApiKey {
  full_key: string;
}

export interface CreateApiKeyRequest {
  name: string;
  scopes: string[];
  expires_at?: string | null;
}

export interface CreateApiKeyResponse {
  key: string;
  info: ApiKey;
}

export function useApiKeys() {
  const [loading, setLoading] = useState(false);
  const { user, session } = useAuth();

  // Create an authenticated fetch request
  const authenticatedFetch = useCallback(async (url: string, options: RequestInit = {}) => {
    // Use the Supabase JWT token for authentication
    // This works with the backend's bootstrap authentication system
    // which accepts JWT tokens for API key creation
    const token = session?.access_token;
    
    if (!token) {
      throw new Error('You must be logged in to perform this action. Please sign in and try again.');
    }

    if (!user) {
      throw new Error('User session is not available. Please refresh the page and try again.');
    }

    const fetchFn = createAuthenticatedFetch(token);
    return fetchFn(url, options);
  }, [session, user]);

  // Fetch user's API keys
  const fetchApiKeys = useCallback(async (): Promise<ApiKey[]> => {
    if (!user || !session) return [];
    
    setLoading(true);
    try {
      const apiUrl = getApiBaseUrl();
      const response = await authenticatedFetch(`${apiUrl}${API_ENDPOINTS.API_KEYS}`);
      const apiKeys: ApiKey[] = await response.json();
      return apiKeys;
    } catch (error) {
      console.error('Error fetching API keys:', error);
      if (error instanceof ApiRequestError) {
        // Log specific API errors for debugging but don't throw
        // Fetching API keys is not critical and should fail gracefully
        console.error('API Error:', error.apiError);
        if (error.apiError.error === 'invalid_api_key' || error.apiError.error === 'expired_jwt') {
          // JWT token might be expired, but don't break the UI
          console.warn('Authentication token may be expired. API key creation may fail.');
        }
      }
      return [];
    } finally {
      setLoading(false);
    }
  }, [user, session, authenticatedFetch]);

  // Create a new API key
  const createApiKey = useCallback(async (name: string, scopes: string[] = ['read'], expiresAt?: string | null): Promise<ApiKeyWithSecret | null> => {
    if (!user) throw new Error('Must be logged in to create API keys');
    
    setLoading(true);
    try {
      const apiUrl = getApiBaseUrl();
      const requestBody: CreateApiKeyRequest = {
        name: name || 'Unnamed Key',
        scopes,
        expires_at: expiresAt,
      };
      
      const response = await authenticatedFetch(
        `${apiUrl}${API_ENDPOINTS.API_KEYS}`,
        {
          method: 'POST',
          body: JSON.stringify(requestBody),
        }
      );
      
      const result: CreateApiKeyResponse = await response.json();
      
      return {
        ...result.info,
        full_key: result.key
      } as ApiKeyWithSecret;
    } catch (error) {
      console.error('Error creating API key:', error);
      
      // Handle specific API errors with user-friendly messages
      if (error instanceof ApiRequestError) {
        switch (error.apiError.error) {
          case 'missing_authentication':
          case 'invalid_jwt':
          case 'expired_jwt':
            throw new Error('Authentication failed. Please sign out and sign in again to refresh your session.');
          case 'configuration_error':
            throw new Error('Server configuration error. Please contact support.');
          case 'invalid_scope':
            throw new Error(`Invalid permission scope: ${error.apiError.message}`);
          case 'database_error':
            throw new Error('Database error occurred. Please try again later.');
          default:
            throw new Error(error.apiError.message || 'Failed to create API key. Please try again.');
        }
      }
      
      // Handle network and other errors
      if (error instanceof Error) {
        if (error.message.includes('fetch')) {
          throw new Error('Network error. Please check your connection and try again.');
        }
        throw error;
      }
      
      throw new Error('An unexpected error occurred. Please try again.');
    } finally {
      setLoading(false);
    }
  }, [user, authenticatedFetch]);

  // Delete an API key
  const deleteApiKey = useCallback(async (keyId: string): Promise<void> => {
    if (!user) throw new Error('Must be logged in to delete API keys');
    
    setLoading(true);
    try {
      const apiUrl = getApiBaseUrl();
      await authenticatedFetch(
        `${apiUrl}${API_ENDPOINTS.API_KEYS}?id=${keyId}`,
        {
          method: 'DELETE',
        }
      );
    } catch (error) {
      console.error('Error deleting API key:', error);
      // Re-throw ApiRequestError to preserve error details
      if (error instanceof ApiRequestError) {
        throw new Error(error.apiError.message);
      }
      throw error;
    } finally {
      setLoading(false);
    }
  }, [user, authenticatedFetch]);

  // Update an API key (name, scopes, etc.)
  const updateApiKey = useCallback(async (keyId: string, updates: { name?: string; scopes?: string[]; is_active?: boolean }): Promise<void> => {
    if (!user) throw new Error('Must be logged in to update API keys');
    
    setLoading(true);
    try {
      const apiUrl = getApiBaseUrl();
      await authenticatedFetch(
        `${apiUrl}${API_ENDPOINTS.API_KEYS}?id=${keyId}`,
        {
          method: 'PATCH',
          body: JSON.stringify(updates),
        }
      );
    } catch (error) {
      console.error('Error updating API key:', error);
      // Re-throw ApiRequestError to preserve error details
      if (error instanceof ApiRequestError) {
        throw new Error(error.apiError.message);
      }
      throw error;
    } finally {
      setLoading(false);
    }
  }, [user, authenticatedFetch]);

  return {
    loading,
    fetchApiKeys,
    createApiKey,
    deleteApiKey,
    updateApiKey
  };
}