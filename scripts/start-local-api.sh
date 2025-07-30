#!/bin/bash

# Start Local API Development Server
# This script starts the Vercel development server for local API testing

set -e

echo "🚀 Starting Carp API Local Development Server..."

# Check if vercel CLI is installed
if ! command -v vercel &> /dev/null; then
    echo "❌ Vercel CLI not found. Installing..."
    npm install -g vercel
fi

# Check if we're in the right directory
if [ ! -f "vercel.json" ]; then
    echo "❌ vercel.json not found. Make sure you're in the project root directory."
    exit 1
fi

# Load environment variables
if [ -f ".env.local" ]; then
    echo "📄 Loading environment variables from .env.local"
    set -o allexport
    source .env.local
    set +o allexport
else
    echo "⚠️  .env.local not found. Using default environment."
fi

# Verify critical environment variables
if [ -z "$SUPABASE_URL" ]; then
    echo "❌ SUPABASE_URL is not set. Please check your .env.local file."
    exit 1
fi

if [ -z "$SUPABASE_ANON_KEY" ]; then
    echo "❌ SUPABASE_ANON_KEY is not set. Please check your .env.local file."
    exit 1
fi

echo "✅ Environment variables loaded successfully"

# Build the Rust functions first
echo "🔨 Building Rust API functions..."
just build-native || {
    echo "❌ Build failed. Make sure you have just installed and the project builds."
    exit 1
}

echo "🌐 Starting Vercel development server..."
echo "📡 API will be available at: http://localhost:3307"
echo "🔍 Test endpoints:"
echo "   Health: http://localhost:3307/api/health"
echo "   Search: http://localhost:3307/api/v1/agents/search"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Start Vercel dev server with environment variables
vercel dev --port 3307
