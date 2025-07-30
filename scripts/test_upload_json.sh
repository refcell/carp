#!/bin/bash

# Test upload with JSON payload directly to API endpoint
API_URL="https://api.carp.refcell.org/api/v1/agents/upload"
API_KEY="carp_test_1234_abcd_efgh"

# Read the test agent file
CONTENT=$(cat test-upload-agent.md)

# Create JSON payload
JSON_PAYLOAD=$(cat <<EOF
{
  "name": "test-upload-agent",
  "description": "A test agent for testing the upload functionality",
  "content": $(echo "$CONTENT" | jq -R -s '.'),
  "version": "1.0.0",
  "tags": ["test", "upload", "demo"],
  "homepage": "https://example.com",
  "repository": "https://github.com/user/test-agent",
  "license": "MIT"
}
EOF
)

echo "Testing upload with JSON payload..."
echo "Payload: $JSON_PAYLOAD"
echo ""

response=$(curl -s -w "%{http_code}" -X POST \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d "$JSON_PAYLOAD" \
  "$API_URL")

http_code=$(echo "$response" | tail -c 4)
body=$(echo "$response" | sed '$s/...$//')

echo "HTTP Code: $http_code"
echo "Response: $body"

if [ "$http_code" -eq 200 ] || [ "$http_code" -eq 201 ]; then
    echo "✓ Upload successful"
else
    echo "✗ Upload failed"
fi