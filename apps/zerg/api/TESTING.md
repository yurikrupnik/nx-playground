# Testing with AppState - Avoiding Breaking Changes

## Problem

When we added Redis to `AppState`, all existing tests broke because they only provided a database connection:

```rust
// ❌ This broke when Redis was added
let app_state = AppState::new(db);
```

## Solution: Builder Pattern + FromRef

We implemented two patterns from the terran API to make tests resilient to AppState changes:

### 1. Builder Pattern (`AppStateBuilder`)

The builder pattern allows flexible construction of AppState without breaking existing code when new fields are added:

```rust
// ✅ Works today with DB + Redis
let state = AppStateBuilder::new()
    .with_db(db)
    .with_redis_mock().await
    .build();

// ✅ In the future, adding MongoDB won't break this
let state = AppStateBuilder::new()
    .with_db(db)
    .with_redis_mock().await
    // .with_mongo(mongo)  <- Future addition
    .build();
```

### 2. FromRef Pattern (Partial State Extraction)

Handlers can extract only the dependencies they need:

```rust
// Before: Handler needs full AppState
pub async fn list_users(
    State(state): State<AppState>,
    // ...
) -> Result<Json<Value>> {
    let users = User::find().all(state.db()).await?;
    // ...
}

// After: Handler extracts only what it needs
pub async fn list_users(
    State(db): State<Arc<DatabaseConnection>>,  // ✅ Only DB!
    // ...
) -> Result<Json<Value>> {
    let users = User::find().all(&db).await?;
    // ...
}
```

**Benefits:**
- Handlers that don't use Redis won't break when Redis is added
- Each handler declares exactly what it needs
- More modular and testable

## How to Use in Tests

### Unit Tests (with MockDatabase)

```rust
async fn setup_mock_state() -> AppState {
    let db = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([/* ... */])
        .into_connection();

    AppStateBuilder::new()
        .with_db(db)
        .with_redis_mock().await  // ✅ Mock Redis
        .build()
}

#[tokio::test]
async fn test_something() {
    let state = setup_mock_state().await;
    // ... test code
}
```

### Integration Tests (with testcontainers)

```rust
// tests/common/mod.rs already configured
impl TestDb {
    pub async fn new() -> Self {
        let connection = Database::connect(&connection_string).await?;
        db::run_migrations(&connection).await?;

        let app_state = AppStateBuilder::new()
            .with_db(connection)
            .with_redis_mock().await  // ✅ Mock Redis
            .build();

        Self { container, app_state, connection_string }
    }
}

#[tokio::test]
async fn test_with_real_db() {
    let test_db = TestDb::new().await;
    let state = test_db.state();
    // ... test code
}
```

### Tests That Actually Use Redis

For tests that need real Redis functionality, use testcontainers:

```rust
use testcontainers_modules::redis::Redis;

#[tokio::test]
async fn test_with_real_redis() {
    let redis_container = Redis::default().start().await?;
    let redis_port = redis_container.get_host_port_ipv4(6379).await?;
    let redis_url = format!("redis://localhost:{}", redis_port);

    let client = redis::Client::open(redis_url)?;
    let redis_manager = ConnectionManager::new(client).await?;

    let state = AppStateBuilder::new()
        .with_db(db)
        .with_redis(redis_manager)  // ✅ Real Redis
        .build();

    // ... test code that uses Redis
}
```

## Future-Proofing

When adding new fields to AppState (e.g., MongoDB, message queue), follow this pattern:

1. **Add field to AppState:**
   ```rust
   pub struct AppState {
       db: Arc<DatabaseConnection>,
       redis: ConnectionManager,
       mongo: Option<MongoClient>,  // New field
   }
   ```

2. **Add builder method:**
   ```rust
   impl AppStateBuilder {
       pub fn with_mongo(mut self, mongo: MongoClient) -> Self {
           self.mongo = Some(mongo);
           self
       }

       pub async fn with_mongo_mock(self) -> Self {
           // Create mock MongoClient
           self.with_mongo(mock_client)
       }
   }
   ```

3. **Add FromRef implementation (optional):**
   ```rust
   impl axum::extract::FromRef<AppState> for MongoClient {
       fn from_ref(state: &AppState) -> Self {
           state.mongo.clone()
       }
   }
   ```

4. **Existing tests don't break!** They'll use the mock automatically.

## Summary

✅ **Tests are now resilient to AppState changes**
✅ **Clear pattern for mocking dependencies**
✅ **Handlers can extract only what they need (FromRef)**
✅ **Easy to add new dependencies in the future**

This pattern is battle-tested in the terran API and follows Axum's best practices.
