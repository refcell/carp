# Carp API Performance Optimizations

This document outlines the backend optimizations implemented to improve performance for agent queries, specifically for latest and trending agents endpoints.

## Optimizations Implemented

### 1. **Dedicated Latest/Trending Endpoints**

**New Endpoints:**
- `GET /api/v1/agents/latest?limit=10` - Optimized latest agents endpoint
- `GET /api/v1/agents/trending?limit=10` - Optimized trending agents endpoint

**Benefits:**
- ✅ Eliminates over-fetching (frontend was loading ALL agents to show 5-10)
- ✅ Server-side sorting using database indexes
- ✅ Reduced payload size (minimal fields only)
- ✅ HTTP caching headers for CDN/browser caching

### 2. **Search Endpoint Optimizations**

**Changes to `/api/v1/agents/search`:**
- ✅ Added explicit `is_public=true` filter to use optimal indexes
- ✅ Expanded search to include `author_name` and `tags` fields
- ✅ Added caching headers (`Cache-Control: max-age=30`)
- ✅ Leverages existing `idx_agents_public_downloads` index

### 3. **Advanced Trending Algorithm**

**Database Function:**
- ✅ `calculate_trending_score()` function using logarithmic download scale
- ✅ Recency bonus for newly created agents (7 days)
- ✅ Activity bonus for recently updated agents (3 days)

**Materialized View:**
- ✅ `trending_agents_mv` pre-calculated trending rankings
- ✅ Indexed by trending score for optimal performance
- ✅ Fallback to regular table if materialized view unavailable

### 4. **Caching Strategy**

**HTTP Cache Headers:**
- Latest agents: 60 seconds (frequent updates expected)
- Trending agents: 300 seconds (materialized view allows longer cache)
- Search results: 30 seconds (balance freshness vs performance)

### 5. **Database Index Utilization**

**Existing Indexes Used:**
- `idx_agents_public_created` - Latest agents endpoint
- `idx_agents_public_downloads` - Search and trending fallback
- `idx_trending_agents_mv_score` - Trending materialized view

## Performance Improvements Expected

### **Latency Improvements:**
- **Latest agents**: ~70% faster (index-optimized query vs full table scan + client sort)
- **Trending agents**: ~80% faster (materialized view vs complex sorting)
- **Search**: ~30% faster (better index utilization + caching)

### **Throughput Improvements:**
- **Reduced database load**: Materialized view eliminates complex calculations per request
- **CDN cacheable**: HTTP headers enable edge caching
- **Smaller payloads**: Optimized response structures reduce network overhead

### **Resource Utilization:**
- **Memory**: Reduced frontend memory usage (no client-side sorting of large datasets)
- **CPU**: Database handles sorting using optimized indexes
- **Network**: Smaller response payloads reduce bandwidth usage

## Usage Examples

### Frontend Integration

```typescript
// Before: Loading all agents then sorting client-side
const allAgents = await fetchAllAgents();
const latest = allAgents.sort(...).slice(0, 10);

// After: Direct endpoint calls
const latest = await fetch('/api/v1/agents/latest?limit=10');
const trending = await fetch('/api/v1/agents/trending?limit=10');
```

### API Responses

```json
// GET /api/v1/agents/latest
{
  "agents": [
    {
      "name": "example-agent",
      "version": "1.0.0",
      "description": "Example agent",
      "author_name": "user123",
      "created_at": "2025-01-31T10:00:00Z",
      "updated_at": "2025-01-31T10:00:00Z",
      "download_count": 42,
      "tags": ["ai", "helper"]
    }
  ],
  "cached_at": "2025-01-31T10:05:00Z"
}
```

## Monitoring & Maintenance

### **Materialized View Refresh**
The trending materialized view should be refreshed periodically:

```sql
-- Refresh trending agents (run every 5-10 minutes)
SELECT public.refresh_trending_agents();
```

### **Performance Metrics to Monitor**
- Endpoint response times (target: <200ms for cached, <500ms for uncached)
- Cache hit rates (target: >80% for trending, >60% for latest)
- Database query execution times
- Materialized view refresh duration

## Future Optimizations

### **Potential Enhancements:**
1. **Redis caching layer** for sub-100ms response times
2. **Background refresh jobs** for materialized views
3. **GraphQL endpoint** to eliminate N+1 profile queries
4. **CDN integration** for global edge caching
5. **Query result compression** for large result sets

## Deployment Notes

1. **Database Migration**: Run the trending score migration before deploying
2. **Vercel Functions**: New endpoints auto-deploy with the API
3. **DNS/Routing**: No changes needed (uses existing API gateway)
4. **Monitoring**: Update dashboards to include new endpoints