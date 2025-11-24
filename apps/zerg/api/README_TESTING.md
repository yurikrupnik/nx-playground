# Testing Guide

## Running Tests

### Option 1: Using the test script (automatically cleans up containers)
```bash
./test.sh
```

Or with specific test filters:
```bash
./test.sh --test integration_tests -- --test-threads=1
```

### Option 2: Manual testing
```bash
# Run all tests
cargo test

# Run integration tests (need serial execution)
cargo test --test integration_tests -- --test-threads=1

# Run specific test file
cargo test --test car_api_test
```

**After running tests manually, clean up containers:**
```bash
docker rm $(docker ps -aq) -f
```

## Test Architecture

### Shared Container Pattern
Tests use a shared container pattern to avoid creating hundreds of containers:
- **3 containers total** per test run (Postgres, MongoDB, Redis)
- Containers are created once and reused across all tests
- Each test gets clean database state (data is truncated between tests)
- Seed data is automatically re-inserted after cleanup

### Test Types

1. **Integration Tests** (`integration_tests.rs`)
   - Use real databases via testcontainers
   - Must run serially: `--test-threads=1`
   - Seed data: 3 users, 5 todos, 3 authors, 5 books, 5 projects

2. **API Tests** (`car_api_test.rs`, `bike_api_test.rs`)
   - Use real databases via testcontainers
   - Can run in parallel
   - Clean state between tests

3. **Unit Tests** (`author_handlers_test.rs`, etc.)
   - Use mock databases
   - Fast and don't need Docker
   - Can run in parallel

## Troubleshooting

### Containers not cleaning up
If you see many containers in Docker Desktop after tests:
```bash
# Clean all containers
docker rm $(docker ps -aq) -f

# Or use the test script which cleans up automatically
./test.sh
```

### Tests failing with data mismatches
Run integration tests serially:
```bash
cargo test --test integration_tests -- --test-threads=1
```

### Container accumulation during development
While developing, containers will accumulate after each test run. Use:
```bash
# Quick cleanup
./test.sh

# Or manual cleanup
docker rm $(docker ps -aq) -f
```
