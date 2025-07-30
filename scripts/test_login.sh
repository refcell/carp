#!/bin/bash

API_BASE_URL="https://api.carp.refcell.org"
USERNAME="${USERNAME:-your_username}"
PASSWORD="${PASSWORD:-your_password}"

echo "Testing login endpoint..."
response=$(curl -s -w "%{http_code}" -X POST \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}" \
  "$API_BASE_URL/api/v1/auth/login")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

if [ "$http_code" -eq 200 ]; then
    echo "✓ Login successful"
    echo "Response: $body"
else
    echo "✗ Login failed (HTTP $http_code)"
    echo "Response: $body"
    exit 1
fi