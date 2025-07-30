/**
 * API Configuration for Carp application
 */

// Get the API base URL based on environment
export const getApiBaseUrl = (): string => {
  // In Vite, environment variables are accessed via import.meta.env
  // and must be prefixed with VITE_ to be available in the browser
  const apiUrl = import.meta.env.VITE_API_URL;
  
  // If a custom API URL is set, use it
  if (apiUrl) {
    return apiUrl;
  }
  
  // For both development and production with Vercel, 
  // API functions are available at the same origin via rewrites
  // This works because Vercel handles /api/* routing to serverless functions
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
    const headers = {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${authToken}`,
      ...options.headers,
    };

    const response = await fetch(url, {
      ...options,
      headers,
    });

    if (!response.ok) {
      let errorData: ApiError;
      try {
        errorData = await response.json();
      } catch {
        errorData = {
          error: 'unknown_error',
          message: `Request failed with status ${response.status}`,
        };
      }
      throw new ApiRequestError(response.status, errorData);
    }

    return response;
  };
};