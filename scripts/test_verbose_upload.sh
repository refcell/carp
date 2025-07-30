#!/bin/bash

echo "Creating test agent file..."
mkdir -p /tmp/test-upload

cat > /tmp/test-upload/test-agent.md << 'EOF'
---
name: test-upload-verbose
description: Test agent for verbose upload debugging
version: "1.0.0"
tags: ["test", "debug"]
---

# Test Upload Agent

This is a test agent to debug the upload functionality.
EOF

echo "Testing CLI upload with verbose output and fake API key..."
echo ""

# Test with verbose output
/Users/andreasbigger/carp/target/release/carp upload \
  --directory /tmp/test-upload \
  --api-key carp_test_abcd_efgh_ijkl \
  --verbose 2>&1

echo ""
echo "Upload test completed."