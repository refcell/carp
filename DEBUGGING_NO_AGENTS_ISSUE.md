# Debugging "No Agents Found" Issue

## Changes Made for Debugging

### 1. Added Extensive Debug Logging
Both `latest.rs` and `trending.rs` now include debug logging that will show:
- Whether environment variables are present
- Database connection status
- Query responses and errors
- Parsed data counts

### 2. Fixed Field Mapping Issues
- Removed confusing `serde(rename)` attributes
- Structs now use exact database field names
- Added defaults for potentially missing fields

### 3. Added Authorization Headers
- Both endpoints now include `Authorization: Bearer` header
- This ensures proper authentication with Supabase

### 4. Created Migration for Missing Data
- `20250802100000_populate_author_name.sql` - Populates empty author_name fields

## What to Check in Deployment

### 1. Check Vercel Function Logs
Look for these debug messages:
- `[DEBUG] SUPABASE_URL present: true/false`
- `[DEBUG] Test query response: ...`
- `[DEBUG] Response body length: ...`
- `[ERROR] Database query failed: ...`

### 2. Verify Environment Variables
In Vercel dashboard, ensure these are set:
- `SUPABASE_URL` - Should be your Supabase project URL
- `SUPABASE_ANON_KEY` or `SUPABASE_SERVICE_ROLE_KEY`

### 3. Test API Endpoints Directly
```bash
# Test latest agents
curl -v https://your-deployment.vercel.app/api/v1/agents/latest

# Test trending agents  
curl -v https://your-deployment.vercel.app/api/v1/agents/trending

# Check response headers for errors
```

### 4. Common Issues and Solutions

#### Issue: Empty Database
**Symptom**: `[DEBUG] Empty response from database`
**Solution**: 
- Run migrations in production
- Ensure some agents have `is_public = true`
- Check that `current_version` is not null

#### Issue: Missing Environment Variables
**Symptom**: `[ERROR] Database not configured`
**Solution**: Add SUPABASE_URL and SUPABASE_ANON_KEY in Vercel

#### Issue: RLS Policies Blocking Access
**Symptom**: Query returns 200 but empty array
**Solution**: 
- Use service role key instead of anon key
- Update RLS policies to allow anonymous reads

#### Issue: Field Parsing Errors
**Symptom**: `[ERROR] Failed to parse agents`
**Solution**: 
- Check that all required fields exist in database
- Run the populate_author_name migration

## Quick Test Script

Create this as a Vercel function to test database connection:

```typescript
export default async function handler(req, res) {
  const url = process.env.SUPABASE_URL;
  const key = process.env.SUPABASE_ANON_KEY;
  
  res.json({
    env_vars_present: !!(url && key),
    url_prefix: url?.substring(0, 30),
    timestamp: new Date().toISOString()
  });
}
```

## Next Steps

1. Deploy these changes
2. Check Vercel function logs for debug output
3. Run the test queries above
4. If still no data, check:
   - Database actually has agents with is_public=true
   - Migrations have been applied
   - RLS policies allow anonymous access