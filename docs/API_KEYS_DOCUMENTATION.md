# API Keys System Documentation

## Overview

The API keys system provides secure programmatic access to the Carp API. This document covers the database schema, security model, and implementation patterns.

**Note**: The API keys table was originally created in migration `20250729100000_add_api_keys_table.sql` and enhanced in `20250729185700_enhance_api_keys_table.sql` with improved security features, validation, and functionality.

## Database Schema

### Table: `public.api_keys`

The `api_keys` table stores hashed API keys that users can generate for programmatic access to the Carp platform.

#### Columns

| Column | Type | Constraints | Description |
|--------|------|-------------|-------------|
| `id` | UUID | PRIMARY KEY, NOT NULL, DEFAULT gen_random_uuid() | Unique identifier for the API key record |
| `user_id` | UUID | NOT NULL, REFERENCES auth.users(id) ON DELETE CASCADE | User who owns this API key |
| `name` | TEXT | NULLABLE | Optional human-readable name/description for the key |
| `key_hash` | TEXT | NOT NULL, UNIQUE | SHA-256 hash of the API key (never store plaintext) |
| `key_prefix` | TEXT | NOT NULL, UNIQUE | First 8+ chars of key for identification (e.g., "carp_k_12345678...") |
| `last_used_at` | TIMESTAMP WITH TIME ZONE | NULLABLE | Track when the key was last used |
| `expires_at` | TIMESTAMP WITH TIME ZONE | NULLABLE | Optional expiration date |
| `is_active` | BOOLEAN | NOT NULL, DEFAULT true | Allow users to disable keys without deleting |
| `scopes` | TEXT[] | DEFAULT '{}' | Array of permissions/scopes for the key (future use) |
| `created_at` | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT now() | When the key was created |
| `updated_at` | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT now() | When the key was last modified |

#### Constraints

- `unique_api_key_hash`: Ensures key_hash is unique across all users
- `unique_api_key_prefix`: Ensures key_prefix is unique across all users for easier identification
- `unique_user_key_name`: Ensures users can't have duplicate key names (if provided)
- `idx_api_keys_user_name_not_null`: Partial unique index allowing multiple NULL names per user

## Security Model

### Key Format

API keys follow the format: `carp_k_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX`

- **Prefix**: `carp_k_` (6 characters)
- **Random Part**: 32+ alphanumeric characters
- **Total Length**: 38+ characters

### Storage Security

1. **Never store plaintext keys**: Only SHA-256 hashes are stored in the database
2. **Key prefix storage**: First 8-16 characters stored for UI identification
3. **Unique constraints**: Both hash and prefix must be globally unique
4. **Secure functions**: Database functions use `SECURITY DEFINER` for controlled access

### Row Level Security (RLS)

All API key operations are protected by RLS policies:

- **SELECT**: Users can only view their own API keys
- **INSERT**: Users can only create API keys for themselves
- **UPDATE**: Users can only modify their own API keys  
- **DELETE**: Users can only delete their own API keys

### Rate Limiting

- Maximum 10 active API keys per user
- Enforced at the database level via trigger validation

## Database Functions

### `public.verify_api_key(key_hash_param TEXT)`

Securely verifies an API key and returns user information.

**Returns**: 
```sql
TABLE (
  user_id UUID, 
  key_id UUID, 
  scopes TEXT[],
  is_valid BOOLEAN
)
```

**Usage**: Called by API endpoints to authenticate requests.

### `public.update_api_key_last_used(key_hash_param TEXT)`

Updates the `last_used_at` timestamp for successful authentications.

**Returns**: `BOOLEAN` (true if key was found and updated)

### `public.deactivate_expired_api_keys()`

Utility function to mark expired keys as inactive.

**Returns**: `INTEGER` (number of keys deactivated)

**Usage**: Can be called periodically to clean up expired keys.

### `public.validate_api_key_format()`

Trigger function that validates API key format and business rules:

- Ensures key_hash is a valid SHA-256 hash (64+ characters)
- Validates key_prefix format (`carp_k_` + alphanumeric)
- Checks expiration dates are in the future
- Enforces the 10 active keys per user limit

## Implementation Patterns

### Key Generation (Rust)

```rust
use rand::Rng;
use sha2::{Digest, Sha256};

pub fn generate_api_key() -> ApiKey {
    // Generate 32 random alphanumeric characters
    let mut rng = rand::thread_rng();
    let random_part: String = (0..32)
        .map(|_| {
            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            chars[rng.gen_range(0..chars.len())] as char
        })
        .collect();

    let full_key = format!("carp_k_{}", random_part);
    
    // Generate SHA-256 hash
    let mut hasher = Sha256::new();
    hasher.update(full_key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());
    
    let key_prefix = full_key.chars().take(16).collect::<String>();

    ApiKey { full_key, key_hash, key_prefix }
}
```

### Authentication Middleware

```rust
pub async fn authenticate_api_key(
    key: &str,
    api_key_manager: &ApiKeyManager,
) -> Result<Option<Uuid>, Box<dyn std::error::Error>> {
    if let Some(key_info) = api_key_manager.verify_api_key(key).await? {
        Ok(key_info.user_id)
    } else {
        Ok(None)
    }
}
```

### Database Operations

Use the provided `ApiKeyManager` struct for all database operations. It handles:

- Secure key creation with proper hashing
- Key verification and user lookup
- Last used timestamp updates
- Key listing with sensitive data exclusion
- Key deactivation

## Migration Files

### Original Migration: `20250729100000_add_api_keys_table.sql`

Creates the basic API keys system including:
- Table schema with core columns and constraints
- RLS policies
- Basic indexes for performance
- Core functions (`validate_api_key`, `update_api_key_last_used`, `generate_api_key`)

### Enhancement Migration: `20250729185700_enhance_api_keys_table.sql`

Enhances the existing API keys system with:
- Improved column constraints (nullable name field)
- Additional security validation triggers
- Enhanced functions with better error handling
- Additional indexes for performance
- Better key format validation
- User key limits (max 10 active keys per user)
- Documentation comments

### Rollback Migration: `20250729185701_rollback_api_keys_enhancements.sql`

Reverts the enhancements while preserving the original API keys table:
- Removes enhanced functions and triggers
- Drops additional constraints and indexes
- Restores original function implementations
- Preserves existing data and core functionality

## Usage Examples

### Creating an API Key

```rust
let (full_key, prefix) = api_key_manager.create_api_key(
    user_id,
    Some("My CLI Key".to_string()),
    None, // No expiration
    "user-auth-token"
).await?;

// ⚠️ Store full_key securely - it's only shown once!
```

### Verifying an API Key

```rust
if let Some(user_id) = authenticate_api_key(&provided_key, &api_key_manager).await? {
    // Key is valid, proceed with authenticated request
} else {
    // Invalid key, return 401 Unauthorized
}
```

### Listing User's API Keys

```rust
let keys = api_key_manager.list_user_api_keys(auth_token).await?;
// Returns ApiKeyListItem[] with sensitive data excluded
```

### Deactivating an API Key

```rust
api_key_manager.deactivate_api_key(key_id, auth_token).await?;
```

## Security Best Practices

1. **Never log API keys**: Ensure keys are not logged in plaintext
2. **Use HTTPS only**: API keys should only be transmitted over secure connections
3. **Implement rate limiting**: Additional rate limiting beyond the 10-key limit
4. **Monitor usage**: Track `last_used_at` for security monitoring
5. **Expire old keys**: Consider implementing automatic expiration policies
6. **Rotate regularly**: Encourage users to rotate keys periodically
7. **Secure storage**: Client applications should store keys securely (keychain, env vars, etc.)

## Frontend Integration

The system integrates with the existing Supabase authentication:

```typescript
// React hook for API key management
const { data: apiKeys, error } = useQuery({
  queryKey: ['api-keys'],
  queryFn: async () => {
    const { data, error } = await supabase
      .from('api_keys')
      .select('id,name,key_prefix,is_active,last_used_at,expires_at,created_at')
      .order('created_at', { ascending: false });
    
    if (error) throw error;
    return data;
  }
});
```

## Performance Considerations

### Indexes

The migration creates optimized indexes for common query patterns:

- `idx_api_keys_user_id`: For fetching user's keys
- `idx_api_keys_key_hash`: For authentication lookups (most critical)
- `idx_api_keys_key_prefix`: For prefix-based identification
- `idx_api_keys_active`: For filtering active keys
- `idx_api_keys_expires_at`: For expiration queries
- `idx_api_keys_created_at`: For chronological ordering

### Query Optimization

- The `verify_api_key` function is optimized for the authentication hot path
- Hash-based lookups provide O(1) key verification
- RLS policies are efficiently indexed on `user_id`

## Monitoring and Maintenance

### Periodic Cleanup

Consider running the cleanup function periodically:

```sql
SELECT public.deactivate_expired_api_keys();
```

### Monitoring Queries

```sql
-- Active keys per user
SELECT user_id, COUNT(*) as active_keys 
FROM api_keys 
WHERE is_active = true 
GROUP BY user_id 
ORDER BY active_keys DESC;

-- Recently used keys
SELECT key_prefix, last_used_at, user_id 
FROM api_keys 
WHERE last_used_at > now() - interval '7 days'
ORDER BY last_used_at DESC;

-- Expired but still active keys
SELECT key_prefix, expires_at, user_id 
FROM api_keys 
WHERE is_active = true AND expires_at < now();
```

## Testing

See `/Users/andreasbigger/carp/examples/api_key_usage.rs` for comprehensive test examples including:

- Key generation testing
- Hash consistency verification
- Uniqueness validation
- Format validation

## Future Enhancements

1. **Scopes System**: Implement fine-grained permissions using the `scopes` column
2. **Key Rotation**: Automated key rotation capabilities
3. **Usage Analytics**: Track API usage per key for billing/monitoring
4. **Key Templates**: Pre-configured key types with specific permissions
5. **Audit Logging**: Detailed audit trail for key operations