#!/bin/bash

# Test script to validate API key listing functionality
# This script tests both JWT and API key authentication methods

API_BASE_URL="https://api.carp.refcell.org"
JWT_TOKEN="${JWT_TOKEN:-your_jwt_token_here}"
API_KEY="${API_KEY:-your_api_key_here}"

echo "=== API Key Listing Validation Test ==="
echo ""

# Test 1: JWT Authentication (should fail according to current implementation)
echo "Test 1: Testing with JWT Authentication (POST method uses this, but GET doesn't)"
echo "URL: $API_BASE_URL/api/v1/auth/api-keys"
echo "Method: GET"
echo "Auth: Bearer JWT"
echo ""

response=$(curl -s -w "%{http_code}" -X GET \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  "$API_BASE_URL/api/v1/auth/api-keys")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

echo "HTTP Status: $http_code"
echo "Response Body: $body"
echo ""

if [ "$http_code" -eq 200 ]; then
    echo "✓ JWT authentication works for listing API keys"
else
    echo "✗ JWT authentication failed for listing API keys (HTTP $http_code)"
    echo "This might be expected if the endpoint requires API key auth"
fi

echo ""
echo "---"
echo ""

# Test 2: API Key Authentication (current implementation should use this)
echo "Test 2: Testing with API Key Authentication"
echo "URL: $API_BASE_URL/api/v1/auth/api-keys"
echo "Method: GET"
echo "Auth: X-API-Key header"
echo ""

response=$(curl -s -w "%{http_code}" -X GET \
  -H "X-API-Key: $API_KEY" \
  -H "Content-Type: application/json" \
  "$API_BASE_URL/api/v1/auth/api-keys")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

echo "HTTP Status: $http_code"
echo "Response Body: $body"
echo ""

if [ "$http_code" -eq 200 ]; then
    echo "✓ API key authentication works for listing API keys"
else
    echo "✗ API key authentication failed for listing API keys (HTTP $http_code)"
fi

echo ""
echo "---"
echo ""

# Test 3: No Authentication (should fail)
echo "Test 3: Testing with No Authentication (should fail)"
echo "URL: $API_BASE_URL/api/v1/auth/api-keys"
echo "Method: GET"
echo "Auth: None"
echo ""

response=$(curl -s -w "%{http_code}" -X GET \
  -H "Content-Type: application/json" \
  "$API_BASE_URL/api/v1/auth/api-keys")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

echo "HTTP Status: $http_code"
echo "Response Body: $body"
echo ""

if [ "$http_code" -eq 401 ] || [ "$http_code" -eq 403 ]; then
    echo "✓ No authentication properly rejected (HTTP $http_code)"
else
    echo "✗ No authentication should have been rejected but got HTTP $http_code"
fi

echo ""
echo "=== Test Summary ==="
echo ""
echo "According to the code analysis:"
echo "- POST /api/v1/auth/api-keys should use JWT authentication (frontend users)"
echo "- GET /api/v1/auth/api-keys should use API key authentication"
echo "- The current implementation has a bug where it uses SUPABASE_SERVICE_ROLE_KEY"
echo "  for both 'apikey' and 'Authorization' headers instead of the user's JWT token"
echo ""
echo "Expected behavior:"
echo "- JWT auth should work for creating API keys (POST)"
echo "- API key auth should work for listing API keys (GET)"
echo "- The Supabase request should use the authenticated user's JWT token"
echo "  in the Authorization header, not the service role key"