#!/bin/bash

API_BASE_URL="https://api.carp.refcell.org"
JWT_TOKEN="${JWT_TOKEN:-your_jwt_token_here}"

echo "Testing create API key endpoint..."
response=$(curl -s -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"test-key\"}" \
  "$API_BASE_URL/api/v1/auth/api-keys")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

if [ "$http_code" -eq 201 ] || [ "$http_code" -eq 200 ]; then
    echo "✓ Create API key successful"
    echo "Response: $body"
else
    echo "✗ Create API key failed (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi