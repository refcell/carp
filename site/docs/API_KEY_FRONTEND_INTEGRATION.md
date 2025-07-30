# Frontend API Key Integration Documentation

## Overview

This document describes the frontend integration for the API key management system, including the fixes applied to resolve the "prefix" vs "key_prefix" backend bug and the complete implementation of the frontend API client.

## Issues Fixed

### 1. Backend API Integration
- **Problem**: Frontend was using direct Supabase database calls instead of the backend API
- **Solution**: Updated `useApiKeys` hook to call `/api/v1/auth/api-keys` endpoint
- **Files Modified**: `/site/src/hooks/useApiKeys.tsx`

### 2. Request Format Mismatch
- **Problem**: Frontend was sending database-specific fields instead of API-expected format
- **Solution**: Updated to send `{ name, scopes, expires_at }` format as expected by backend
- **Files Modified**: `/site/src/hooks/useApiKeys.tsx`

### 3. Authentication Header
- **Problem**: Missing Authorization header required by backend API
- **Solution**: Added proper `Authorization: Bearer <token>` header
- **Files Modified**: `/site/src/lib/api-config.ts`, `/site/src/hooks/useApiKeys.tsx`

### 4. Response Handling
- **Problem**: Frontend expected different response format than backend provides
- **Solution**: Updated to handle `{ key, info }` response structure from backend
- **Files Modified**: `/site/src/hooks/useApiKeys.tsx`

### 5. Error Handling
- **Problem**: Generic error messages without backend error details
- **Solution**: Proper error parsing and display of backend error messages
- **Files Modified**: `/site/src/lib/api-config.ts`, `/site/src/components/ApiKeyManagementModal.tsx`

### 6. User Experience Improvements
- **Problem**: Limited scope selection and poor error feedback
- **Solution**: Added scopes selection UI and better error messages
- **Files Modified**: `/site/src/components/ApiKeyManagementModal.tsx`

## Architecture

### API Configuration (`/site/src/lib/api-config.ts`)
```typescript
// Centralized API configuration
export const getApiBaseUrl = (): string => {
  return process.env.NODE_ENV === 'development' 
    ? process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8000'
    : process.env.NEXT_PUBLIC_API_URL || 'https://api.getcarp.dev';
};

// Authenticated fetch wrapper with proper error handling
export const createAuthenticatedFetch = (authToken: string) => {
  // Returns a fetch function with proper headers and error handling
}
```

### API Hook (`/site/src/hooks/useApiKeys.tsx`)
```typescript
export function useApiKeys() {
  // Provides:
  // - fetchApiKeys(): Promise<ApiKey[]>
  // - createApiKey(name, scopes, expires_at): Promise<ApiKeyWithSecret>
  // - deleteApiKey(keyId): Promise<void>
  // - updateApiKey(keyId, updates): Promise<void>
}
```

### UI Component (`/site/src/components/ApiKeyManagementModal.tsx`)
```typescript
export function ApiKeyManagementModal({ children }) {
  // Features:
  // - Scopes selection UI
  // - Real-time error display
  // - Secure key display (show/hide)
  // - Copy to clipboard functionality
}
```

## API Endpoints

### GET /api/v1/auth/api-keys
**Purpose**: List user's API keys
**Authentication**: Bearer token (Supabase JWT or existing API key)
**Response**: `ApiKey[]`

### POST /api/v1/auth/api-keys
**Purpose**: Create new API key
**Authentication**: Bearer token
**Request Body**:
```json
{
  "name": "My API Key",
  "scopes": ["read", "write"],
  "expires_at": "2024-12-31T23:59:59Z" // optional
}
```
**Response**:
```json
{
  "key": "carp_xxxxxxxx_xxxxxxxx_xxxxxxxx",
  "info": {
    "id": "uuid",
    "name": "My API Key",
    "prefix": "carp_xxxxxxxx",
    "scopes": ["read", "write"],
    "is_active": true,
    "last_used_at": null,
    "expires_at": "2024-12-31T23:59:59Z",
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### DELETE /api/v1/auth/api-keys?id={keyId}
**Purpose**: Delete an API key
**Authentication**: Bearer token
**Response**: 204 No Content

### PATCH /api/v1/auth/api-keys?id={keyId}
**Purpose**: Update API key (name, scopes, active status)
**Authentication**: Bearer token
**Request Body**: 
```json
{
  "name": "Updated Name",
  "scopes": ["read"],
  "is_active": false
}
```

## Bootstrap Authentication Problem

### The Challenge
There's a circular dependency problem: the API requires an API key for authentication, but we need to call the API to create the first API key.

### Current Solution
The frontend currently uses the Supabase JWT token as a fallback authentication method:

```typescript
const authenticatedFetch = useCallback(async (url: string, options: RequestInit = {}) => {
  // Use Supabase JWT token as fallback for initial API key creation
  const token = session?.access_token;
  
  if (!token) {
    throw new Error('No authentication token available');
  }

  const fetchFn = createAuthenticatedFetch(token);
  return fetchFn(url, options);
}, [session]);
```

### Recommended Solutions

#### Option 1: Hybrid Authentication (Current Implementation)
- Accept both API keys and Supabase JWT tokens in the backend
- Use Supabase JWT for initial API key creation
- Use API keys for subsequent operations
- **Pros**: Solves bootstrap problem, maintains security
- **Cons**: Dual authentication system complexity

#### Option 2: Admin-Created Initial Keys
- Admin creates initial API key via backend admin interface
- Users receive their first API key through secure channel (email, etc.)
- All subsequent operations use API keys
- **Pros**: Pure API key authentication
- **Cons**: Requires admin intervention, complex user onboarding

#### Option 3: OAuth Flow with API Key Exchange
- User authenticates via OAuth (GitHub, etc.)
- Backend automatically creates initial API key during first OAuth login
- Frontend exchanges OAuth token for API key
- **Pros**: Automated, secure
- **Cons**: Requires OAuth integration in backend

### Recommended Implementation
The current hybrid approach (Option 1) is recommended because:
1. It solves the bootstrap problem elegantly
2. Maintains existing Supabase authentication flow
3. Allows gradual migration to pure API key authentication
4. Provides flexibility for different authentication contexts

## Environment Configuration

### Development
```env
# .env.local
NEXT_PUBLIC_API_URL=http://localhost:8000
```

### Production
```env
# .env.production
NEXT_PUBLIC_API_URL=https://api.getcarp.dev
```

## Security Considerations

1. **API Key Storage**: Keys are only shown once during creation. Frontend should not persist them.
2. **Token Security**: Supabase JWT tokens are used as fallback authentication.
3. **HTTPS Only**: All API calls must use HTTPS in production.
4. **Scope Principle**: Users should select minimal required scopes.
5. **Error Handling**: Avoid exposing sensitive error details to end users.

## Testing

### Manual Testing Checklist
- [ ] Create API key with different scopes
- [ ] List existing API keys
- [ ] Copy API key to clipboard
- [ ] Delete API key
- [ ] Handle network errors gracefully
- [ ] Handle invalid authentication
- [ ] Verify scopes display correctly

### API Integration Tests
```typescript
// Example test structure
describe('API Key Integration', () => {
  test('should create API key with correct format', async () => {
    const apiKey = await createApiKey('Test Key', ['read']);
    expect(apiKey.full_key).toMatch(/^carp_/);
    expect(apiKey.info.scopes).toEqual(['read']);
  });
});
```

## Deployment Considerations

1. **Environment Variables**: Ensure `NEXT_PUBLIC_API_URL` is set correctly
2. **CORS Configuration**: Backend must allow requests from frontend domain
3. **Rate Limiting**: Consider implementing rate limiting for API key operations
4. **Monitoring**: Monitor API key creation/deletion patterns for security

## Future Enhancements

1. **Key Rotation**: Implement automatic key rotation functionality
2. **Usage Analytics**: Track API key usage and display to users
3. **Webhooks**: Allow users to configure webhooks for key events
4. **Team Management**: Support for shared API keys within organizations
5. **Audit Logging**: Detailed audit trail for all API key operations

## Troubleshooting

### Common Issues

#### "No authentication token available"
- **Cause**: User not logged in or session expired
- **Solution**: Ensure user is authenticated via Supabase

#### "API Error: Invalid or expired API key"
- **Cause**: Backend can't verify the provided token
- **Solution**: Check token format and backend authentication logic

#### "Failed to create API key"
- **Cause**: Various backend validation errors
- **Solution**: Check backend logs for specific error details

#### Network/CORS Errors
- **Cause**: Frontend can't reach backend API
- **Solution**: Verify API URL and CORS configuration

### Debug Mode
To enable detailed API debugging:
```typescript
// In api-config.ts
const DEBUG_API = process.env.NODE_ENV === 'development';

if (DEBUG_API) {
  console.log('API Request:', url, options);
  console.log('API Response:', response);
}
```

## Migration Guide

If migrating from direct Supabase calls to this API integration:

1. **Update imports**: Replace Supabase client imports with new hook
2. **Update method calls**: Use new method signatures
3. **Update error handling**: Handle new error types
4. **Update UI components**: Use new data structures
5. **Test thoroughly**: Verify all functionality works end-to-end

## Support

For issues with this integration:
1. Check browser console for detailed error messages
2. Verify environment configuration
3. Test backend API endpoints directly
4. Review this documentation for common solutions