#!/bin/bash

# Script to test Anthropic API key with curl

# Load API key from environment
source .env 

echo "Testing Anthropic API with key: ${ANTHROPIC_API_KEY}"
echo "Key length: ${#ANTHROPIC_API_KEY}"

# Make a request to the Anthropic API using curl
curl -v https://api.anthropic.com/v1/messages \
  -H "content-type: application/json" \
  -H "x-api-key: ${ANTHROPIC_API_KEY}" \
  -H "anthropic-version: 2023-01-01" \
  -d '{
    "model": "claude-3-sonnet-20240229",
    "max_tokens": 256,
    "messages": [
      {
        "role": "user",
        "content": "Hello! How are you today?"
      }
    ]
  }'