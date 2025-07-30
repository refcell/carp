#!/usr/bin/env python3

import subprocess
import sys
import os

def test_upload():
    # Create test agent
    test_dir = "/tmp/carp-test-upload"
    os.makedirs(test_dir, exist_ok=True)
    
    agent_content = """---
name: test-batch-upload
description: Test agent for batch upload debugging
version: "1.0.0"
tags: ["test", "debug", "batch"]
---

# Test Batch Upload Agent

This is a test agent to debug the batch upload functionality."""

    with open(f"{test_dir}/test-agent.md", "w") as f:
        f.write(agent_content)
    
    print("Created test agent file")
    
    # Run CLI with expect-like input simulation
    cli_path = "/Users/andreasbigger/carp/cli/target/release/carp"
    cmd = [
        cli_path, "upload",
        "--directory", test_dir,
        "--api-key", "carp_test_abcd_efgh_ijkl",
        "--verbose"
    ]
    
    print(f"Running command: {' '.join(cmd)}")
    print("=" * 50)
    
    # Use pexpect-like approach with subprocess
    try:
        # Run the command with input simulation
        process = subprocess.Popen(
            cmd,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=0
        )
        
        # Send "All agents" selection (typically option 1 or 2)
        output, _ = process.communicate(input="2\n")
        
        print("CLI Output:")
        print(output)
        print("=" * 50)
        print(f"Exit code: {process.returncode}")
        
    except Exception as e:
        print(f"Error running CLI: {e}")

if __name__ == "__main__":
    test_upload()