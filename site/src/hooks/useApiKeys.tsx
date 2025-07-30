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
    // For now, we'll use the Supabase JWT token as a fallback
    // In a full implementation, this should use an existing API key
    // This creates a bootstrap problem that needs to be addressed separately
    const token = session?.access_token;
    
    if (!token) {
      throw new Error('No authentication token available');
    }

    const fetchFn = createAuthenticatedFetch(token);
    return fetchFn(url, options);
  }, [session]);

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
        // Handle specific API errors if needed
        console.error('API Error:', error.apiError);
      }
      return [];
    } finally {
      setLoading(false);
    }
  }, [user, session, getApiBaseUrl, authenticatedFetch]);

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
      // Re-throw ApiRequestError to preserve error details
      if (error instanceof ApiRequestError) {
        throw new Error(error.apiError.message);
      }
      throw error;
    } finally {
      setLoading(false);
    }
  }, [user, getApiBaseUrl, authenticatedFetch]);

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
  }, [user, getApiBaseUrl, authenticatedFetch]);

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
  }, [user, getApiBaseUrl, authenticatedFetch]);

  return {
    loading,
    fetchApiKeys,
    createApiKey,
    deleteApiKey,
    updateApiKey
  };
}