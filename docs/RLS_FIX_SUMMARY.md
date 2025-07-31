# RLS Policy Fix for Agent Upload Issue

## Problem Summary

Users with valid API keys were getting this error when uploading agents:
```
Error: API error (400): Database error: {"code":"42501","details":null,"hint":null,"message":"new row violates row-level security policy for table \"agents\""}
```

PostgreSQL error code 42501 indicates insufficient privileges due to Row-Level Security (RLS) policy violations.

## Root Cause Analysis

### The Issue
The `agents` table had RLS policies that only worked with JWT authentication:

```sql
CREATE POLICY "Users can create their own agents" 
ON public.agents 
FOR INSERT 
WITH CHECK (auth.uid() = user_id);
```

### Why It Failed
- **JWT Authentication**: Sets `auth.uid()` in Supabase auth context
- **API Key Authentication**: Uses service role without setting `auth.uid()`
- **Result**: `auth.uid()` returns NULL for API key requests, failing the `auth.uid() = user_id` check

### Architecture Mismatch
- API keys validated via `validate_api_key()` database function
- User context established in application layer 
- RLS policies couldn't access this application-layer context
- Service role headers didn't bypass RLS policies with explicit `auth.uid()` checks

## Solution Implemented

### 1. Fixed RLS Policies (`20250731000000_fix_agents_rls_for_api_keys.sql`)

Updated agents table policies to handle both authentication methods:

```sql
CREATE POLICY "Users can create their own agents" 
ON public.agents 
FOR INSERT 
WITH CHECK (
  -- Allow JWT authentication (web UI)
  (auth.uid() IS NOT NULL AND auth.uid() = user_id)
  OR
  -- Allow service role (API key authentication)
  (auth.role() = 'service_role')
);
```

**Key Changes:**
- Added `auth.role() = 'service_role'` condition
- Service role requests bypass RLS since application already validated API key
- Maintains security: only properly authenticated requests reach this point

### 2. Fixed User Synchronization (`20250731000001_ensure_api_key_users_exist.sql`)

**Problems Found:**
- `sync_jwt_user()` tried to insert into non-existent `users` table
- API key users weren't synced to `profiles` table
- Foreign key constraints required users to exist

**Solutions:**
- Created `sync_jwt_user_fixed()` function using correct `profiles` table
- Created `sync_api_key_user()` function for API key users
- Both functions handle upserts safely with conflict resolution

### 3. Updated Application Code

**In `shared/auth.rs`:**
- Fixed `sync_jwt_user()` to use database function
- Added `sync_api_key_user()` function

**In `shared/middleware.rs`:**
- Added user sync for both JWT and API key authentication
- Non-fatal sync errors (logged but don't block operations)

## Why This Fix Works

### Security Maintained
- **JWT Users**: Full RLS protection with `auth.uid()` context
- **API Key Users**: Application-layer validation before service role database access
- **Service Role**: Only used after API key validation confirms user ownership

### Compatibility
- **Existing JWT Authentication**: No changes, continues to work
- **New API Key Authentication**: Now properly supported
- **Database Integrity**: Foreign key constraints maintained

### Error Scenarios Handled
- API key users without profiles: Auto-created
- JWT users without profiles: Auto-synced
- Missing auth.users entries: Logged but non-fatal
- Network/sync failures: Logged but don't block operations

## Testing the Fix

1. **Apply Migrations:**
   ```bash
   # Apply the RLS policy fix
   psql -f site/supabase/migrations/20250731000000_fix_agents_rls_for_api_keys.sql
   
   # Apply the user sync fix
   psql -f site/supabase/migrations/20250731000001_ensure_api_key_users_exist.sql
   ```

2. **Test API Key Upload:**
   ```bash
   # Set API key
   export CARP_API_KEY="your_api_key_here"
   
   # Test upload (should now work)
   carp upload agent-name --description "Test agent"
   ```

3. **Verify Database State:**
   ```sql
   -- Check RLS policies
   SELECT schemaname, tablename, policyname, cmd, qual 
   FROM pg_policies 
   WHERE tablename = 'agents';
   
   -- Check user sync functions exist
   SELECT proname FROM pg_proc WHERE proname LIKE '%sync%user%';
   ```

## Files Modified

- `/site/supabase/migrations/20250731000000_fix_agents_rls_for_api_keys.sql` (NEW)
- `/site/supabase/migrations/20250731000001_ensure_api_key_users_exist.sql` (NEW)  
- `/shared/auth.rs` (MODIFIED)
- `/shared/middleware.rs` (MODIFIED)

## Long-term Considerations

1. **API Key User Creation**: Ensure API keys are only created for users with proper auth.users entries
2. **Monitoring**: Track sync failures in production
3. **Performance**: Consider caching user sync status
4. **Security Audit**: Regular review of service role usage patterns