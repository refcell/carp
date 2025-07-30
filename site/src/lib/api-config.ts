/**
 * API Configuration for Carp application
 */

// Get the API base URL based on environment
export const getApiBaseUrl = (): string => {
  // Check if we're in development mode
  if (process.env.NODE_ENV === 'development') {
    // For local development, API functions are served from the same origin
    // as the frontend via Vercel's local development server
    return process.env.NEXT_PUBLIC_API_URL || '';
  }
  
  // For production, API functions are served from the same origin
  // No separate API domain needed with Vercel serverless functions
  return process.env.NEXT_PUBLIC_API_URL || '';
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