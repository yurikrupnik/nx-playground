#!/bin/bash
set -e

PORT=${PORT:-8080}
POSTMAN_DIR="postman"
OPENAPI_FILE="$POSTMAN_DIR/openapi.json"
OUTPUT_FILE="$POSTMAN_DIR/zerg-api-generated.postman_collection.json"
CONFIG_FILE="$POSTMAN_DIR/portman-config.json"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

cd "$(dirname "$0")/.."

echo -e "${BLUE}🔧 Generating Postman collection with tests from OpenAPI spec${NC}"

# Step 1: Fetch OpenAPI spec
echo -e "${BLUE}📡 Fetching OpenAPI spec from http://localhost:$PORT/api-docs/openapi.json${NC}"

if ! curl -sf http://localhost:$PORT/health > /dev/null 2>&1; then
  echo -e "${RED}❌ Server is not running on port $PORT${NC}"
  echo -e "${YELLOW}Start the server first: cargo run${NC}"
  exit 1
fi

if ! curl -sf http://localhost:$PORT/api-docs/openapi.json -o "$OPENAPI_FILE"; then
  echo -e "${RED}❌ Failed to fetch OpenAPI spec${NC}"
  exit 1
fi

echo -e "${GREEN}✅ OpenAPI spec fetched${NC}"

# Step 2: Generate Postman collection with Portman
echo -e "${BLUE}🚀 Running Portman to generate collection with tests${NC}"

if npx portman --local "$OPENAPI_FILE" \
  --output "$OUTPUT_FILE" \
  --portmanConfigFile "$CONFIG_FILE" \
  --baseUrl "http://localhost:$PORT"; then

  echo -e "${GREEN}✅ Postman collection generated with tests${NC}"

  # Reorder collection for proper test flow
  echo -e "${BLUE}🔄 Reordering requests for optimal test flow...${NC}"
  node scripts/reorder-collection.js

  echo -e "${BLUE}📊 Collection includes:${NC}"
  echo -e "${GREEN}   - Status code validation${NC}"
  echo -e "${GREEN}   - Schema validation${NC}"
  echo -e "${GREEN}   - Content type checks${NC}"
  echo -e "${GREEN}   - Variable assignments${NC}"
else
  echo -e "${RED}❌ Failed to generate Postman collection${NC}"
  exit 1
fi
