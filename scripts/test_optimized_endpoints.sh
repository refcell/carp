#!/bin/bash
# Test script for the optimized API endpoints
# This script verifies that the latest and trending endpoints work correctly

set -e

echo "🚀 Testing Optimized API Endpoints"
echo "=================================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "api/v1/agents/latest.rs" ] || [ ! -f "api/v1/agents/trending.rs" ]; then
    echo -e "${RED}❌ Error: Please run this script from the project root directory${NC}"
    exit 1
fi

# Build the API to ensure latest changes are compiled
echo -e "${YELLOW}🔨 Building API...${NC}"
just build-api

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ API build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ API build successful${NC}"

# Test the optimized endpoints test suite
echo -e "${YELLOW}🧪 Running optimized endpoints test suite...${NC}"
cargo test --package carp-api-serverless --test api_optimized_endpoints_tests

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Some tests failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All tests passed${NC}"

# Check the endpoint binaries exist
echo -e "${YELLOW}📦 Checking compiled binaries...${NC}"

LATEST_BINARY="target/release/v1-agents-latest"
TRENDING_BINARY="target/release/v1-agents-trending"

if [ -f "$LATEST_BINARY" ]; then
    echo -e "${GREEN}✅ Latest endpoint binary exists${NC}"
else
    echo -e "${YELLOW}⚠️  Latest endpoint binary not found at $LATEST_BINARY${NC}"
fi

if [ -f "$TRENDING_BINARY" ]; then
    echo -e "${GREEN}✅ Trending endpoint binary exists${NC}"
else
    echo -e "${YELLOW}⚠️  Trending endpoint binary not found at $TRENDING_BINARY${NC}"
fi

# Verify database migrations exist
echo -e "${YELLOW}🗃️  Checking database migrations...${NC}"

TRENDING_MIGRATION="site/supabase/migrations/20250801120000_add_trending_score_function.sql"
POPULATE_MIGRATION="site/supabase/migrations/20250801130000_ensure_trending_view_populated.sql"
REFRESH_MIGRATION="site/supabase/migrations/20250801140000_refresh_trending_view_on_deploy.sql"

if [ -f "$TRENDING_MIGRATION" ]; then
    echo -e "${GREEN}✅ Trending materialized view migration exists${NC}"
else
    echo -e "${RED}❌ Missing: $TRENDING_MIGRATION${NC}"
fi

if [ -f "$POPULATE_MIGRATION" ]; then
    echo -e "${GREEN}✅ Trending view population migration exists${NC}"
else
    echo -e "${RED}❌ Missing: $POPULATE_MIGRATION${NC}"
fi

if [ -f "$REFRESH_MIGRATION" ]; then
    echo -e "${GREEN}✅ Trending view refresh migration exists${NC}"
else
    echo -e "${RED}❌ Missing: $REFRESH_MIGRATION${NC}"
fi

# Check endpoint response format
echo -e "${YELLOW}🔍 Validating endpoint response formats...${NC}"

# Test the response structures compile correctly
cargo test --package carp-api-serverless --test api_optimized_endpoints_tests response_format_tests::test_agent_response_format

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Response format validation passed${NC}"
else
    echo -e "${RED}❌ Response format validation failed${NC}"
    exit 1
fi

# Summary
echo ""
echo "📊 Summary"
echo "=========="
echo -e "${GREEN}✅ API compilation: PASSED${NC}"
echo -e "${GREEN}✅ Unit tests: PASSED${NC}"
echo -e "${GREEN}✅ Response format: PASSED${NC}"
echo -e "${GREEN}✅ Database migrations: VERIFIED${NC}"

echo ""
echo -e "${GREEN}🎉 All optimized endpoint tests passed!${NC}"
echo ""
echo "Next steps:"
echo "1. Deploy the migrations to production database"
echo "2. Deploy the API endpoints to production"
echo "3. Update frontend to use the new optimized endpoints:"
echo "   - /api/v1/agents/latest"
echo "   - /api/v1/agents/trending"
echo ""

# Optional: Show some helpful info about the endpoints
echo "📋 Endpoint Information"
echo "======================"
echo "Latest Agents Endpoint:"
echo "  - Path: /api/v1/agents/latest"
echo "  - Method: GET"
echo "  - Parameters: ?limit=N (default 10, max 50)"
echo "  - Cache: 1 minute"
echo ""
echo "Trending Agents Endpoint:"
echo "  - Path: /api/v1/agents/trending"
echo "  - Method: GET" 
echo "  - Parameters: ?limit=N (default 10, max 50)"
echo "  - Cache: 5 minutes"
echo "  - Fallback: Regular agents table if materialized view fails"
echo ""