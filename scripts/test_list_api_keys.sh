#!/bin/bash

API_BASE_URL="https://api.carp.refcell.org"
JWT_TOKEN="${JWT_TOKEN:-your_jwt_token_here}"

echo "Testing list API keys endpoint..."
response=$(curl -s -w "%{http_code}" -X GET \
  -H "Authorization: Bearer $JWT_TOKEN" \
  "$API_BASE_URL/api/v1/auth/api-keys")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

if [ "$http_code" -eq 200 ]; then
    echo "✓ List API keys successful"
    echo "Response: $body"
else
    echo "✗ List API keys failed (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi