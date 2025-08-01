#!/bin/bash

# Test script to verify API endpoints work correctly

echo "Testing API endpoints..."
echo "========================"

# Test latest agents endpoint
echo -e "\n1. Testing /api/v1/agents/latest endpoint:"
curl -s -X GET "http://localhost:3000/api/v1/agents/latest?limit=5" | jq '.'

# Test trending agents endpoint  
echo -e "\n2. Testing /api/v1/agents/trending endpoint:"
curl -s -X GET "http://localhost:3000/api/v1/agents/trending?limit=5" | jq '.'

# Test CORS headers
echo -e "\n3. Testing CORS headers on /api/v1/agents/latest:"
curl -s -I -X GET "http://localhost:3000/api/v1/agents/latest" | grep -i "access-control"

echo -e "\n4. Testing OPTIONS preflight on /api/v1/agents/latest:"
curl -s -I -X OPTIONS "http://localhost:3000/api/v1/agents/latest" | grep -i "access-control"

echo -e "\nDone!"