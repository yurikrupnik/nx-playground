#!/bin/bash
# Test runner with automatic container cleanup

set -e

echo "🧪 Running tests..."

# Run tests
cargo test --package zerg_api "$@"

# Cleanup containers after tests
echo ""
echo "🧹 Cleaning up test containers..."
docker ps -aq --filter "label=org.testcontainers=true" | xargs -r docker rm -f 2>/dev/null || true
docker ps -aq --filter "ancestor=postgres:17-alpine" --filter "ancestor=mongo:7" --filter "ancestor=redis:7-alpine" | xargs -r docker rm -f 2>/dev/null || true

echo "✅ Cleanup complete!"
