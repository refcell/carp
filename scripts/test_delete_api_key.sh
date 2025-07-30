#!/bin/bash

API_BASE_URL="https://api.carp.refcell.org"
API_KEY="${API_KEY:-your_api_key_here}"
KEY_ID="${KEY_ID:-key_uuid_here}"

echo "Testing delete API key endpoint..."
response=$(curl -s -w "%{http_code}" -X DELETE \
  -H "X-API-Key: $API_KEY" \
  "$API_BASE_URL/api/v1/auth/api-keys?id=$KEY_ID")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

if [ "$http_code" -eq 200 ] || [ "$http_code" -eq 204 ]; then
    echo "✓ Delete API key successful"
    echo "Response: $body"
else
    echo "✗ Delete API key failed (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi