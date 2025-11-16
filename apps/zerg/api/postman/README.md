# Postman/Newman API Tests

This directory contains automated Postman test generation and Newman test automation for the Zerg API.

## 📂 Files

### Generated Files (gitignored)
- **`openapi.json`** - Auto-generated from `/api-docs/openapi.json` endpoint
- **`zerg-api-generated.postman_collection.json`** - Auto-generated collection with test scripts

### Configuration
- **`portman-config.json`** - Portman configuration for test generation
- **`local.postman_environment.json`** - Local development environment (port 8080)
- **`ci.postman_environment.json`** - CI/CD environment

## ✨ Automated Test Generation

This setup uses **Portman** to automatically generate Postman collections with test scripts from your OpenAPI spec. **No manual maintenance required!**

### How It Works:
```
Your Rust Code (utoipa) → OpenAPI Spec → Portman → Postman Collection with Tests → Newman
```

When you change your API:
1. OpenAPI spec updates automatically (utoipa annotations)
2. Collection regenerates with updated endpoints and tests
3. Tests stay in sync with your code

## Automated Test Coverage

Portman automatically generates tests for all endpoints including:

### Contract Tests (Auto-generated)
- ✅ **Status Code Validation** - Ensures correct HTTP status codes (200, 201, 400, etc.)
- ✅ **Schema Validation** - Validates response against OpenAPI schema
- ✅ **Content Type Checks** - Verifies `application/json` headers
- ✅ **Response Time Checks** - Ensures responses under 5 seconds
- ✅ **JSON Body Validation** - Checks response structure

### Variable Management (Auto-configured)
- Automatically extracts `userId` from POST response
- Uses variables in subsequent requests (GET, PUT, DELETE)

### Request Ordering (Smart)
- Creates user first (POST)
- Gets user details (GET by ID)
- Lists users (GET all)
- Updates user (PUT)
- Deletes user (DELETE)

## Quick Commands

```bash
# Generate Postman collection with tests (server must be running)
nx run zerg_api:postman:generate

# Run Newman automated tests (starts server automatically)
nx run zerg_api:test:newman

# Fetch only the OpenAPI spec
nx run zerg_api:openapi:fetch
```

## Running Tests

### Prerequisites

1. **Database**: Ensure PostgreSQL is running:
   ```bash
   docker run -d \
     --name zerg-postgres \
     -e POSTGRES_USER=postgres \
     -e POSTGRES_PASSWORD=postgres \
     -e POSTGRES_DB=zerg_api \
     -p 5432:5432 \
     postgres:16-alpine
   ```

2. **Environment Variables** (optional):
   ```bash
   export DATABASE_URL=postgres://postgres:postgres@localhost:5432/zerg_api
   ```

### Run via Nx (Fully Automated)

```bash
# Run Newman tests (starts server, generates tests, runs them)
nx run zerg_api:test:newman
```

This command will:
1. Start the Zerg API server in the background
2. Wait for it to be ready (30 second timeout with health checks)
3. **Generate Postman collection with tests from OpenAPI spec**
4. Run the Newman collection with all auto-generated tests
5. Generate an HTML report
6. Stop the server automatically

### Run Manually (with server running)

If you prefer to run steps separately:

```bash
# Terminal 1: Start the server
cd apps/zerg/api
cargo run

# Terminal 2: Generate collection with tests
nx run zerg_api:postman:generate

# Terminal 3: Run Newman
npx newman run postman/zerg-api-generated.postman_collection.json \
  -e postman/local.postman_environment.json \
  --reporters cli,htmlextra \
  --reporter-htmlextra-export postman/reports/report.html
```

### Import into Postman Desktop

**Option 1: Direct OpenAPI Import (Recommended for exploration)**
1. Open Postman
2. Click "Import" → "Link"
3. Enter: `http://localhost:8080/api-docs/openapi.json`
4. Select the environment: `local.postman_environment.json`
5. Explore and test endpoints

**Option 2: Import Generated Collection (With test scripts)**
1. Run `nx run zerg_api:postman:generate` (server must be running)
2. Import `postman/zerg-api-generated.postman_collection.json`
3. Import `postman/local.postman_environment.json`
4. Run collection to see auto-generated tests

## Test Results

### Console Output

Newman provides real-time test results in the console with:
- Request details
- Test assertions (pass/fail)
- Response times
- Overall summary

### HTML Report

After running tests, an HTML report is generated at:
```
apps/zerg/api/postman/reports/report.html
```

The report includes:
- Summary statistics
- Detailed test results
- Request/response data
- Response times and sizes
- Charts and visualizations

## CI/CD Integration

The Newman tests can be integrated into CI/CD pipelines:

```yaml
# Example GitHub Actions
- name: Run API Tests
  run: |
    docker-compose up -d postgres
    nx run zerg_api:test:newman
```

The tests are **fully automated**:
- Collection regenerates from OpenAPI spec
- Tests are always in sync with your API
- No manual updates needed

## Customizing Test Generation

Edit `postman/portman-config.json` to customize test generation:

```json
{
  "tests": {
    "contractTests": [
      {
        "statusCode": { "enabled": true }
      },
      {
        "schemaValidation": { "enabled": true }
      }
    ]
  }
}
```

See [Portman documentation](https://github.com/apideck-libraries/portman) for all options.

## How Variables Work

Portman automatically configures variables based on `portman-config.json`:

- `userId` - Extracted from POST `/api/users` response
- Used automatically in subsequent GET, PUT, DELETE requests
- No manual variable management needed

## Tips

1. **Always Fresh Tests**: Re-run `nx run zerg_api:postman:generate` to get latest tests matching your API

2. **Customize Test Rules**: Edit `portman-config.json` to add/remove test types

3. **Request Ordering**: Configured in `orderOfOperations` to ensure logical flow (create → get → update → delete)

4. **Validation Coverage**: Portman generates validation tests for all error responses defined in your OpenAPI spec

## Troubleshooting

### Server Not Starting

If the server fails to start:
- Check that port 3000 is available
- Ensure PostgreSQL is running
- Check database connection settings

### Tests Failing

If tests fail:
- Ensure the server is running and healthy
- Check the database has the latest migrations
- Review the HTML report for detailed error messages

### Permission Denied

If you get a permission error running the script:
```bash
chmod +x apps/zerg/api/scripts/run-newman-tests.sh
```

### Server Won't Start on macOS/Linux

Ensure you have `curl` installed (used for health checks):
```bash
# macOS
brew install curl

# Linux
sudo apt-get install curl
```
