#!/usr/bin/env python3
"""
Basic Claude AI Agent Template

This is a template for creating Claude AI agents.
Customize this file to implement your agent's specific functionality.
"""

import json
import sys
from typing import Dict, Any

class Agent:
    """Basic Claude AI Agent"""
    
    def __init__(self, config: Dict[str, Any] = None):
        """Initialize the agent with optional configuration."""
        self.config = config or {}
        self.name = self.config.get('name', 'Basic Agent')
        self.version = self.config.get('version', '0.1.0')
    
    def process(self, input_data: str) -> str:
        """Process input and return output."""
        # TODO: Implement your agent logic here
        return f"Hello from {self.name}! You said: {input_data}"
    
    def handle_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle a structured request."""
        try:
            input_data = request.get('input', '')
            result = self.process(input_data)
            
            return {
                'success': True,
                'result': result,
                'agent': {
                    'name': self.name,
                    'version': self.version
                }
            }
        except Exception as e:
            return {
                'success': False,
                'error': str(e),
                'agent': {
                    'name': self.name,
                    'version': self.version
                }
            }

def main():
    """Main entry point for the agent."""
    # Load configuration if available
    config = {}
    try:
        with open('config.toml', 'r') as f:
            # Basic TOML parsing (you might want to use a proper TOML library)
            pass
    except FileNotFoundError:
        pass
    
    agent = Agent(config)
    
    if len(sys.argv) > 1:
        # Command line input
        input_data = ' '.join(sys.argv[1:])
        result = agent.process(input_data)
        print(result)
    else:
        # Interactive mode or JSON input
        try:
            line = input()
            request = json.loads(line)
            response = agent.handle_request(request)
            print(json.dumps(response))
        except (EOFError, KeyboardInterrupt):
            pass
        except json.JSONDecodeError:
            # Treat as plain text input
            result = agent.process(line)
            print(result)

if __name__ == '__main__':
    main()
