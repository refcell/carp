# Carp Package Management Database Schema

This document describes the comprehensive database schema implemented for the Carp package management system in Supabase.

## Overview

The schema extends the existing basic agents and profiles tables to provide full package management capabilities including:

- Version control for agents
- Package file storage and metadata
- CLI authentication via API tokens  
- Download tracking and analytics
- User interactions (ratings, follows)
- Advanced search and discovery
- Rate limiting and security

## Database Tables

### Core Tables

#### `profiles`
User profile information (extends existing table)
- `id` - UUID primary key
- `user_id` - Reference to auth.users
- `github_username` - GitHub username
- `display_name` - Display name
- `avatar_url` - Profile picture URL
- `bio` - User biography
- `created_at`, `updated_at` - Timestamps

#### `agents` 
Agent definitions and metadata (extended from existing)
- `id` - UUID primary key
- `user_id` - Reference to auth.users (owner)
- `name` - Unique agent name
- `description` - Agent description
- `definition` - JSONB agent configuration
- `tags` - Array of tags
- `is_public` - Public visibility flag
- `view_count` - View counter
- **NEW FIELDS:**
- `current_version` - Current version string
- `author_name` - Author display name
- `license` - License identifier
- `homepage` - Homepage URL
- `repository` - Repository URL
- `keywords` - Array of keywords for search
- `download_count` - Total download counter
- `latest_version_id` - Reference to latest version
- `readme` - README content
- `created_at`, `updated_at` - Timestamps

#### `agent_versions`
Version history for each agent
- `id` - UUID primary key
- `agent_id` - Reference to agents table
- `version` - Version string (semver)
- `description` - Version description
- `changelog` - Version changelog
- `definition` - JSONB agent configuration for this version
- `package_size` - Package file size in bytes
- `checksum` - Package file checksum
- `download_count` - Version-specific download count
- `is_pre_release` - Pre-release flag
- `yanked` - Yanked/withdrawn flag
- `yanked_reason` - Reason for yanking
- `created_at`, `updated_at` - Timestamps
- **UNIQUE:** (agent_id, version)

#### `agent_packages`
Package file metadata and storage paths
- `id` - UUID primary key
- `version_id` - Reference to agent_versions
- `file_name` - Original filename
- `file_path` - Storage path in bucket
- `content_type` - MIME type
- `file_size` - File size in bytes
- `checksum` - File checksum
- `upload_completed` - Upload completion flag
- `created_at` - Timestamp
- **UNIQUE:** (version_id, file_name)

### Authentication & Access

#### `api_tokens`
CLI authentication tokens
- `id` - UUID primary key
- `user_id` - Reference to auth.users
- `token_name` - User-friendly name
- `token_hash` - Hashed token value
- `token_prefix` - First 8 chars for identification
- `scopes` - Array of permissions (read, write, admin)
- `last_used_at` - Last usage timestamp
- `last_used_ip` - Last usage IP address
- `expires_at` - Optional expiration date
- `is_active` - Active status flag
- `created_at`, `updated_at` - Timestamps

### Analytics & Tracking

#### `download_stats`
Download tracking and analytics
- `id` - UUID primary key
- `agent_id` - Reference to agents
- `version_id` - Reference to agent_versions (optional)
- `package_id` - Reference to agent_packages (optional)
- `user_id` - Reference to auth.users (optional, null for anonymous)
- `ip_address` - Client IP address
- `user_agent` - Client user agent
- `referer` - HTTP referer
- `country_code` - 2-letter country code
- `downloaded_at` - Download timestamp
- `file_size` - Downloaded file size

#### `rate_limits`
API rate limiting tracking
- `id` - UUID primary key
- `identifier` - IP address or user ID
- `endpoint` - API endpoint
- `request_count` - Request count in window
- `window_start` - Time window start
- `created_at` - Timestamp
- **UNIQUE:** (identifier, endpoint, window_start)

### Social Features

#### `user_follows`
User and agent following relationships
- `id` - UUID primary key
- `follower_id` - Reference to auth.users (follower)
- `following_user_id` - Reference to auth.users (followed user, optional)
- `following_agent_id` - Reference to agents (followed agent, optional)
- `created_at` - Timestamp
- **CONSTRAINT:** Either following_user_id OR following_agent_id must be set
- **UNIQUE:** (follower_id, following_user_id), (follower_id, following_agent_id)

#### `agent_ratings`
Agent ratings and reviews
- `id` - UUID primary key
- `agent_id` - Reference to agents
- `user_id` - Reference to auth.users
- `rating` - Integer rating (1-5)
- `review` - Optional review text
- `helpful_count` - Helpfulness counter
- `created_at`, `updated_at` - Timestamps
- **UNIQUE:** (agent_id, user_id)

### System Tables

#### `webhook_events`
Event log for external integrations
- `id` - UUID primary key
- `event_type` - Event type string
- `agent_id` - Reference to agents (optional)
- `version_id` - Reference to agent_versions (optional)
- `user_id` - Reference to auth.users (optional)
- `payload` - JSONB event data
- `processed` - Processing status flag
- `created_at` - Timestamp

## Storage Buckets

### `agent-packages`
Supabase Storage bucket for package files
- **Access:** Private with RLS policies
- **File Size Limit:** 100MB per file
- **Allowed MIME Types:** gzip, tar+gzip, zip
- **Path Structure:** `{user_id}/{agent_name}/{version}/{filename}`

## Database Functions

### Search & Discovery

#### `search_agents(search_query, tags_filter, author_filter, sort_by, sort_order, page_num, page_size)`
Advanced agent search with full-text search, filtering, and pagination
- **Returns:** Agent results with metadata and pagination info
- **Features:** Text search, tag filtering, author filtering, multiple sort options
- **Sort Options:** relevance, downloads, created_at, updated_at, rating, name

#### `get_agent_details(agent_name, agent_author)`
Get detailed agent information including all versions
- **Returns:** Complete agent details with version history
- **Access:** Public agents only

#### `get_popular_tags(limit_count)`
Get most popular tags across all public agents
- **Returns:** Tag names with usage counts
- **Ordering:** By count descending, then alphabetically

#### `get_agent_dependencies(agent_name)`
Extract and return agent dependencies from definition
- **Returns:** Dependency names, version constraints, and types
- **Source:** Parses `dependencies` field in agent definition

### Package Management

#### `create_agent(agent_name, description, author_name, tags, keywords, license, homepage, repository, readme, is_public)`
Create a new agent with metadata
- **Authentication:** Requires user auth or valid API token
- **Validation:** Checks for unique agent names
- **Returns:** Success/error status with agent ID

#### `publish_agent_version(agent_name, version, description, changelog, definition_data, package_data)`
Publish a new version of an agent
- **Authentication:** Requires ownership or API token with write scope
- **Validation:** Checks for unique versions
- **Updates:** Sets as current version, creates package record
- **Returns:** Success/error status with version and package IDs

#### `record_download(agent_name, version_text, user_agent_text, ip_addr)`
Record a package download event
- **Tracking:** Updates download counters, logs analytics
- **Anonymous:** Supports anonymous downloads
- **Returns:** Success boolean

### Authentication

#### `validate_api_token(token_hash)`
Validate and update API token usage
- **Updates:** Last used timestamp and IP address
- **Validation:** Checks active status and expiration
- **Returns:** User ID and scopes for valid tokens

### Analytics & Stats

#### `get_user_agent_stats(target_user_id)`
Get comprehensive statistics for a user's agents
- **Returns:** Total agents, downloads, versions, ratings, etc.
- **Access:** Own stats or specified user

### Utility Functions

#### `refresh_trending_agents()`
Refresh the trending agents materialized view
- **Calculation:** Based on recent downloads, ratings, and activity
- **Usage:** Should be called periodically via cron job

#### `cleanup_old_data()`
Clean up old data to maintain performance
- **Removes:** Old download stats (1 year+), rate limits (1 day+), processed webhooks (30 days+), expired tokens
- **Usage:** Should be called periodically via cron job

#### `check_rate_limit(identifier, endpoint, max_requests, window_minutes)`
Check and enforce API rate limits
- **Parameters:** Configurable request limits and time windows
- **Returns:** Boolean indicating if request is allowed

## Views

### `agent_stats`
Aggregated statistics for all agents
- **Data:** Download counts, version counts, ratings, followers, recent activity
- **Performance:** Pre-computed aggregations for dashboard usage

### `trending_agents` (Materialized View)
Trending agents based on activity score
- **Algorithm:** Weighted score from recent downloads (60%), ratings (30%), review count (10%)
- **Refresh:** Manual via `refresh_trending_agents()` function
- **Usage:** Homepage trending section

## Row Level Security (RLS)

All tables have RLS enabled with comprehensive policies:

### Agent Access
- **Public agents:** Readable by everyone
- **Private agents:** Only readable by owner
- **API tokens:** Support read/write operations with proper scopes

### Ownership-Based Access
- **Profiles:** Users can read all, update own
- **Agents:** Users can CRUD own agents
- **Versions/Packages:** Access follows agent ownership
- **API Tokens:** Users can CRUD own tokens
- **Ratings/Follows:** Users can CRUD own records

### System Tables
- **Download Stats:** Only readable by agent owners, writable by system
- **Rate Limits/Webhooks:** System-only access

## Indexes

### Performance Indexes
- **Full-text search:** GIN index on agent search text
- **Composite indexes:** For common query patterns (public + downloads, public + created_at, etc.)
- **Foreign key indexes:** On all relationship columns
- **Partial indexes:** For active tokens, public agents, non-yanked versions

### Search Optimization
- **Text search:** Uses immutable function for consistent GIN indexing
- **Tag search:** GIN index on tag arrays
- **Keyword search:** GIN index on keyword arrays

## Security Features

### Function Security
- All functions use `SECURITY DEFINER` with `SET search_path = ''`
- Input validation and sanitization
- Proper error handling without information leakage

### API Token Security
- Tokens are hashed before storage
- Scoped permissions (read, write, admin)
- Expiration support
- Usage tracking

### Rate Limiting
- Configurable per-endpoint limits
- IP and user-based tracking
- Automatic cleanup of old records

### Storage Security
- Private bucket with RLS policies
- Path validation ensuring user/agent ownership
- File type and size restrictions

## Migration Files

1. **`20250727123053_initial_schema.sql`** - Original profiles and agents tables
2. **`20250727123210_security_fixes.sql`** - Security improvements for functions
3. **`20250727123232_trigger_fixes.sql`** - Trigger setup and fixes
4. **`20250727134000_package_management_schema.sql`** - Main package management tables and RLS
5. **`20250727134100_storage_and_functions.sql`** - Storage setup and core functions
6. **`20250727134200_utility_functions.sql`** - Utility functions, views, and performance indexes

## API Integration

The schema is designed to support both:
- **Web application:** Direct Supabase client access with RLS
- **CLI tool:** API token-based authentication with function calls
- **External integrations:** Webhook events for notifications

## Monitoring & Maintenance

### Recommended Periodic Tasks
1. **Daily:** Run `cleanup_old_data()` to remove old records
2. **Hourly:** Run `refresh_trending_agents()` to update trending view
3. **Weekly:** Analyze slow queries and add indexes as needed
4. **Monthly:** Review and archive old download statistics

### Performance Monitoring
- Monitor index usage and query performance
- Track storage bucket usage and costs
- Review rate limiting patterns
- Analyze popular search queries for optimization opportunities

This schema provides a robust foundation for a modern package management system with comprehensive security, analytics, and scalability features.