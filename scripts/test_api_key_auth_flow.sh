#!/bin/bash

# Test script to demonstrate the API key authentication flow issue
# This script shows the circular dependency problem in the current implementation

API_BASE_URL="https://localhost:3307"
JWT_TOKEN="${JWT_TOKEN:-your_jwt_token_here}"
API_KEY="${API_KEY:-your_api_key_here}"

echo "=== API Key Authentication Flow Test ==="
echo ""

echo "Current Implementation Analysis:"
echo "- POST /api/v1/auth/api-keys (create) requires JWT authentication"
echo "- GET /api/v1/auth/api-keys (list) requires API key authentication"
echo "- This creates a circular dependency!"
echo ""

echo "Scenario 1: New user with only JWT token (from web login)"
echo "==============================================="
echo ""

# Test 1: Try to list API keys with JWT (should fail in current implementation)
echo "Step 1: Attempt to list API keys with JWT token"
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
echo "Response: $body"
echo ""

if [ "$http_code" -eq 401 ] || [ "$http_code" -eq 403 ]; then
    echo "❌ As expected, JWT cannot list API keys in current implementation"
    echo "   This means new users cannot see their API keys through the web interface!"
else
    echo "✅ JWT authentication works for listing API keys"
fi

echo ""
echo "Step 2: Create first API key with JWT token"
echo "URL: $API_BASE_URL/api/v1/auth/api-keys"
echo "Method: POST"
echo "Auth: Bearer JWT"
echo ""

create_payload='{
  "name": "Test Key",
  "scopes": ["read", "write"],
  "expires_at": null
}'

response=$(curl -s -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "$create_payload" \
  "$API_BASE_URL/api/v1/auth/api-keys")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

echo "HTTP Status: $http_code"
echo "Response: $body"
echo ""

if [ "$http_code" -eq 201 ]; then
    echo "✅ API key creation works with JWT"
    # Extract the API key from response for next test
    NEW_API_KEY=$(echo "$body" | python3 -c "import json,sys; data=json.load(sys.stdin); print(data.get('key', 'not_found'))" 2>/dev/null || echo "not_found")
    echo "   Created API key: ${NEW_API_KEY:0:20}..."
else
    echo "❌ API key creation failed"
    NEW_API_KEY="$API_KEY"  # Fallback to provided API key
fi

echo ""
echo "---"
echo ""

echo "Scenario 2: User with API key (from CLI or after creation)"
echo "======================================================="
echo ""

echo "Step 3: List API keys with the API key"
echo "URL: $API_BASE_URL/api/v1/auth/api-keys"
echo "Method: GET"
echo "Auth: X-API-Key header"
echo ""

response=$(curl -s -w "%{http_code}" -X GET \
  -H "X-API-Key: $NEW_API_KEY" \
  -H "Content-Type: application/json" \
  "$API_BASE_URL/api/v1/auth/api-keys")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

echo "HTTP Status: $http_code"
echo "Response: $body"
echo ""

if [ "$http_code" -eq 200 ]; then
    echo "✅ API key authentication works for listing API keys"
else
    echo "❌ API key authentication failed for listing API keys"
fi

echo ""
echo "=== Problem Summary ==="
echo ""
echo "ISSUE IDENTIFIED:"
echo "1. Web users (with JWT) cannot list their existing API keys"
echo "2. This breaks the user experience for API key management in the frontend"
echo "3. Users who lose their API key cannot see what keys they have"
echo ""
echo "RECOMMENDATION:"
echo "- GET /api/v1/auth/api-keys should accept BOTH JWT and API key authentication"
echo "- This would allow both web users and CLI users to list their keys"
echo "- The current POST (create) + GET (list) inconsistency should be fixed"
echo ""
echo "CURRENT AUTHENTICATION MATRIX:"
echo "┌─────────────────────────────────┬─────────────┬─────────────────┐"
echo "│ Endpoint                        │ JWT Support │ API Key Support │"
echo "├─────────────────────────────────┼─────────────┼─────────────────┤"
echo "│ POST /api/v1/auth/api-keys      │ ✅ Yes      │ ❌ No           │"
echo "│ GET /api/v1/auth/api-keys       │ ❌ No       │ ✅ Yes          │"
echo "│ PUT /api/v1/auth/api-keys       │ ❌ No       │ ✅ Yes          │"
echo "│ DELETE /api/v1/auth/api-keys    │ ❌ No       │ ✅ Yes          │"
echo "└─────────────────────────────────┴─────────────┴─────────────────┘"
echo ""
echo "RECOMMENDED AUTHENTICATION MATRIX:"
echo "┌─────────────────────────────────┬─────────────┬─────────────────┐"
echo "│ Endpoint                        │ JWT Support │ API Key Support │"
echo "├─────────────────────────────────┼─────────────┼─────────────────┤"
echo "│ POST /api/v1/auth/api-keys      │ ✅ Yes      │ ❌ No           │"
echo "│ GET /api/v1/auth/api-keys       │ ✅ Yes      │ ✅ Yes          │"
echo "│ PUT /api/v1/auth/api-keys       │ ✅ Yes      │ ✅ Yes          │"
echo "│ DELETE /api/v1/auth/api-keys    │ ✅ Yes      │ ✅ Yes          │"
echo "└─────────────────────────────────┴─────────────┴─────────────────┘"
