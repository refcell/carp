/**
 * API Configuration for Carp application
 * 
 * Handles environment detection, API URL configuration, and authenticated requests
 * with comprehensive logging for debugging authentication issues.
 */

// Get the API base URL based on environment with improved detection
export const getApiBaseUrl = (): string => {
  // In Vite, environment variables are accessed via import.meta.env
  // and must be prefixed with VITE_ to be available in the browser
  const apiUrl = import.meta.env.VITE_API_URL;
  const isDev = import.meta.env.DEV;
  const mode = import.meta.env.MODE;
  
  console.log('[API Config] Environment detection:', { isDev, mode, apiUrl });
  
  // If a custom API URL is explicitly set, use it
  if (apiUrl) {
    console.log('[API Config] Using custom API URL:', apiUrl);
    return apiUrl;
  }
  
  // Environment-specific defaults with explicit fallbacks
  if (isDev) {
    // Development environment - typically local backend
    const devUrl = 'http://localhost:3000';
    console.log('[API Config] Using development API URL:', devUrl);
    return devUrl;
  }
  
  // Production environment - API at same origin (Vercel setup)
  // This works because Vercel handles /api/* routing to serverless functions
  console.log('[API Config] Using production API URL: same origin');
  return '';
};

// API endpoints
export const API_ENDPOINTS = {
  API_KEYS: '/api/v1/auth/api-keys',
} as const;

/**
 * Create an authenticated fetch request with proper error handling
 */
export interface ApiError {
  error: string;
  message: string;
  details?: any;
}

export class ApiRequestError extends Error {
  public status: number;
  public apiError: ApiError;

  constructor(status: number, apiError: ApiError) {
    super(apiError.message);
    this.status = status;
    this.apiError = apiError;
    this.name = 'ApiRequestError';
  }
}

export const createAuthenticatedFetch = (authToken: string) => {
  return async (url: string, options: RequestInit = {}): Promise<Response> => {
    console.log('[API Config] Making authenticated request to:', url);
    
    const headers = {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${authToken}`,
      ...options.headers,
    };

    try {
      const response = await fetch(url, {
        ...options,
        headers,
      });

      console.log('[API Config] Response status:', response.status, response.statusText);

      if (!response.ok) {
        let errorData: ApiError;
        try {
          errorData = await response.json();
          console.error('[API Config] API error response:', errorData);
        } catch {
          errorData = {
            error: 'unknown_error',
            message: `Request failed with status ${response.status}`,
          };
          console.error('[API Config] Failed to parse error response');
        }
        throw new ApiRequestError(response.status, errorData);
      }

      console.log('[API Config] Request successful');
      return response;
    } catch (error) {
      if (error instanceof ApiRequestError) {
        throw error;
      }
      console.error('[API Config] Network or fetch error:', error);
      throw error;
    }
  };
};