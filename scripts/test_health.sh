#!/bin/bash

API_BASE_URL="https://api.carp.refcell.org"

echo "Testing health endpoint..."
response=$(curl -s -w "%{http_code}" -X GET "$API_BASE_URL/api/health")
http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

if [ "$http_code" -eq 200 ]; then
    echo "✓ Health check passed"
    echo "Response: $body"
else
    echo "✗ Health check failed (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi
