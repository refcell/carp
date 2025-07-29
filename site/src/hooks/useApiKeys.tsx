import { useState, useCallback } from 'react';
import { supabase } from '@/integrations/supabase/client';
import { useAuth } from './useAuth';
import { Tables } from '@/integrations/supabase/types';

export interface ApiKey {
  id: string;
  name: string;
  prefix: string;
  scopes: string[];
  is_active: boolean;
  last_used_at: string | null;
  expires_at: string | null;
  created_at: string;
  updated_at: string;
  user_id: string;
}

export interface ApiKeyWithSecret extends ApiKey {
  full_key: string;
}

export function useApiKeys() {
  const [loading, setLoading] = useState(false);
  const { user } = useAuth();

  // Generate a cryptographically secure API key
  const generateApiKey = useCallback(() => {
    const array = new Uint8Array(32);
    crypto.getRandomValues(array);
    const key = Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
    return `carp_${key}`;
  }, []);

  // Hash an API key for storage (simplified - in production use proper hashing)
  const hashApiKey = useCallback((key: string) => {
    // In production, use bcrypt or similar
    // For now, we'll use a simple hash approach
    return btoa(key).replace(/=/g, '');
  }, []);

  // Get the prefix from a full API key
  const getKeyPrefix = useCallback((key: string) => {
    return key.substring(0, 12); // "carp_" + first 8 characters
  }, []);

  // Fetch user's API keys
  const fetchApiKeys = useCallback(async (): Promise<ApiKey[]> => {
    if (!user) return [];
    
    setLoading(true);
    try {
      const { data, error } = await supabase
        .from('api_keys')
        .select('id, name, prefix, scopes, is_active, last_used_at, expires_at, created_at, updated_at, user_id')
        .eq('user_id', user.id)
        .order('created_at', { ascending: false });
      
      if (error) {
        console.error('Error fetching API keys:', error);
        return [];
      }
      
      return data as ApiKey[];
    } catch (error) {
      console.error('Error fetching API keys:', error);
      return [];
    } finally {
      setLoading(false);
    }
  }, [user]);

  // Create a new API key
  const createApiKey = useCallback(async (name?: string): Promise<ApiKeyWithSecret | null> => {
    if (!user) throw new Error('Must be logged in to create API keys');
    
    setLoading(true);
    try {
      const fullKey = generateApiKey();
      const hashedKey = hashApiKey(fullKey);
      const prefix = getKeyPrefix(fullKey);
      
      const { data, error } = await supabase
        .from('api_keys')
        .insert([{
          name: name || 'Unnamed Key',
          key_hash: hashedKey,
          prefix: prefix,
          user_id: user.id,
          scopes: ['read'], // Default scopes
          is_active: true
        }])
        .select('id, name, prefix, scopes, is_active, last_used_at, expires_at, created_at, updated_at, user_id')
        .single();
      
      if (error) {
        console.error('Error creating API key:', error);
        throw error;
      }
      
      return {
        ...data,
        full_key: fullKey
      } as ApiKeyWithSecret;
    } catch (error) {
      console.error('Error creating API key:', error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [user, generateApiKey, hashApiKey, getKeyPrefix]);

  // Delete an API key
  const deleteApiKey = useCallback(async (keyId: string): Promise<void> => {
    if (!user) throw new Error('Must be logged in to delete API keys');
    
    setLoading(true);
    try {
      const { error } = await supabase
        .from('api_keys')
        .delete()
        .eq('id', keyId)
        .eq('user_id', user.id);
      
      if (error) {
        console.error('Error deleting API key:', error);
        throw error;
      }
    } catch (error) {
      console.error('Error deleting API key:', error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [user]);

  // Update an API key's name
  const updateApiKeyName = useCallback(async (keyId: string, name: string): Promise<void> => {
    if (!user) throw new Error('Must be logged in to update API keys');
    
    setLoading(true);
    try {
      const { error } = await supabase
        .from('api_keys')
        .update({ name })
        .eq('id', keyId)
        .eq('user_id', user.id);
      
      if (error) {
        console.error('Error updating API key:', error);
        throw error;
      }
    } catch (error) {
      console.error('Error updating API key:', error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [user]);

  return {
    loading,
    fetchApiKeys,
    createApiKey,
    deleteApiKey,
    updateApiKeyName
  };
}