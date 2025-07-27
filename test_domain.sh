#!/bin/bash

# Test script for custom domain setup
DOMAIN="api.carp.refcell.org"

echo "Testing custom domain setup for $DOMAIN"
echo "========================================"

# Test DNS resolution
echo "1. Testing DNS resolution..."
nslookup $DOMAIN
echo ""

# Test HTTPS connection
echo "2. Testing HTTPS connection..."
curl -I https://$DOMAIN/health
echo ""

# Test API endpoint
echo "3. Testing API endpoint..."
curl -s "https://$DOMAIN/api/v1/agents/search?q=test" | jq '.' || echo "Response is not JSON"
echo ""

# Test with CLI
echo "4. Testing with CLI..."
cd cli
cargo run -- search test --limit 5

# Send a request to the pull url
PULL_URL=https://$DOMAIN/api/v1/agents/senior-code-reviewer/latest/download
echo "5. Testing pull URL: $PULL_URL"
curl -I $PULL_URL
