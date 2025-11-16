#!/bin/bash
set -e

PORT=${PORT:-8080}
POSTGRES_HOST=${POSTGRES_HOST:-localhost}
POSTGRES_PORT=${POSTGRES_PORT:-5432}
POSTGRES_USER=${POSTGRES_USER:-myuser}
POSTGRES_PASSWORD=${POSTGRES_PASSWORD:-mypassword}
POSTGRES_CONTAINER=${POSTGRES_CONTAINER:-dockers-postgres-1}

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

cd "$(dirname "$0")/.."

# Create unique test database name
TEST_DB="zerg_api_test_$(date +%s)"
TEST_DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}:${POSTGRES_PORT}/${TEST_DB}"

echo -e "${BLUE}🗄️  Creating test database: ${TEST_DB}${NC}"
docker exec -e PGPASSWORD=${POSTGRES_PASSWORD} ${POSTGRES_CONTAINER} psql -U ${POSTGRES_USER} -d mydatabase -c "CREATE DATABASE ${TEST_DB};" 2>/dev/null || {
  echo -e "${RED}❌ Failed to create test database. Make sure PostgreSQL container is running.${NC}"
  exit 1
}

# Cleanup function
cleanup() {
  echo -e "${YELLOW}🛑 Stopping server...${NC}"
  kill $SERVER_PID 2>/dev/null || true

  echo -e "${YELLOW}🗑️  Dropping test database: ${TEST_DB}${NC}"
  docker exec -e PGPASSWORD=${POSTGRES_PASSWORD} ${POSTGRES_CONTAINER} psql -U ${POSTGRES_USER} -d mydatabase -c "DROP DATABASE IF EXISTS ${TEST_DB};" 2>/dev/null || true
}
trap cleanup EXIT

# Start server with test database (migrations run automatically on startup)
echo -e "${BLUE}🚀 Starting Zerg API server with test database...${NC}"
echo -e "${BLUE}   (Migrations will run automatically)${NC}"
DATABASE_URL=${TEST_DATABASE_URL} cargo run --bin zerg_api > /dev/null 2>&1 &
SERVER_PID=$!

# Wait for server to be ready
echo -e "${YELLOW}⏳ Waiting for server to be ready...${NC}"
for i in {1..30}; do
  if curl -sf http://localhost:$PORT/health > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Server is ready!${NC}"
    # Give extra time for migrations to complete
    echo -e "${YELLOW}⏳ Waiting for migrations to complete...${NC}"
    sleep 3
    break
  fi
  if [ $i -eq 30 ]; then
    echo -e "${RED}❌ Server failed to start${NC}"
    exit 1
  fi
  sleep 1
done

# Generate Postman collection with tests from OpenAPI
echo -e "${BLUE}🔧 Generating Postman collection with tests...${NC}"
if ! bash scripts/generate-postman-tests.sh; then
  echo -e "${RED}❌ Failed to generate Postman collection${NC}"
  exit 1
fi

# Run newman tests
echo -e "${BLUE}🧪 Running Newman tests...${NC}"
npx newman run postman/zerg-api-generated.postman_collection.json \
  -e postman/local.postman_environment.json \
  --reporters cli,htmlextra \
  --reporter-htmlextra-export postman/reports/report.html \
  --color on

echo -e "${GREEN}✅ Newman tests completed!${NC}"
echo -e "${BLUE}📊 HTML report: postman/reports/report.html${NC}"
