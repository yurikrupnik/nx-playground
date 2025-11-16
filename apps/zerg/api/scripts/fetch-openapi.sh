#!/bin/bash
set -e

PORT=${PORT:-8080}
OUTPUT_DIR="postman"
OUTPUT_FILE="$OUTPUT_DIR/openapi.json"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

cd "$(dirname "$0")/.."

echo -e "${BLUE}📡 Fetching OpenAPI spec from http://localhost:$PORT/api-docs/openapi.json${NC}"

# Check if server is running
if ! curl -sf http://localhost:$PORT/health > /dev/null 2>&1; then
  echo -e "${RED}❌ Server is not running on port $PORT${NC}"
  echo -e "${YELLOW}Start the server first: cargo run${NC}"
  exit 1
fi

# Fetch OpenAPI spec
if curl -sf http://localhost:$PORT/api-docs/openapi.json -o "$OUTPUT_FILE"; then
  echo -e "${GREEN}✅ OpenAPI spec saved to $OUTPUT_FILE${NC}"
  echo -e "${BLUE}📊 Import this file into Postman for auto-generated collection${NC}"

  # Pretty print the version info
  VERSION=$(cat "$OUTPUT_FILE" | grep -o '"version":"[^"]*"' | cut -d'"' -f4)
  TITLE=$(cat "$OUTPUT_FILE" | grep -o '"title":"[^"]*"' | cut -d'"' -f4)
  echo -e "${GREEN}   API: $TITLE (v$VERSION)${NC}"
else
  echo -e "${RED}❌ Failed to fetch OpenAPI spec${NC}"
  exit 1
fi
