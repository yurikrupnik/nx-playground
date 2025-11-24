# Composition Pattern Quick Reference Guide

## What is the Composition Pattern?

The composition pattern uses small, focused traits that can be combined to create larger capabilities. Instead of one monolithic trait, we have:

```rust
// ❌ OLD WAY: Monolithic trait
trait ApiState {
    fn db(&self) -> &DatabaseConnection;
    fn mongo(&self) -> &Database;
    fn redis(&self) -> &ConnectionManager;
}

// ✅ NEW WAY: Composable traits
trait HasDatabase { fn db(&self) -> &DatabaseConnection; }  // SeaORM
trait HasSqlxPool { fn sqlx_pool(&self) -> &PgPool; }      // sqlx
trait HasMongoDB { fn mongo(&self) -> &Database; }
trait HasRedis { fn redis(&self) -> &ConnectionManager; }
```

## How It Works

### 1. Define Base Traits (`libs/app/src/state.rs`)

```rust
pub trait HasDatabase: Clone + Send + Sync + 'static {
    fn db(&self) -> &DatabaseConnection;
}

pub trait HasSqlxPool: Clone + Send + Sync + 'static {
    fn sqlx_pool(&self) -> &PgPool;
}

pub trait HasMongoDB: Clone + Send + Sync + 'static {
    fn mongo(&self) -> &Database;
}

pub trait HasRedis: Clone + Send + Sync + 'static {
    fn redis(&self) -> &ConnectionManager;
}
```

### 2. API Defines Its Needs (`libs/apis/car/src/state.rs`)

```rust
// Car API only needs MongoDB and Redis
pub trait CarState: HasMongoDB + HasRedis {}

// Blanket implementation: any type with HasMongoDB + HasRedis gets CarState for free
impl<T> CarState for T where T: HasMongoDB + HasRedis {}
```

**That's it! Only 2 lines per API!**

### 3. AppState Implements Base Traits Once (`apps/zerg/api/src/state.rs`)

```rust
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub mongo: Database,
    pub redis: ConnectionManager,
}

// Implement each base trait once
impl HasDatabase for AppState {
    fn db(&self) -> &DatabaseConnection { &self.db }
}

impl HasMongoDB for AppState {
    fn mongo(&self) -> &Database { &self.mongo }
}

impl HasRedis for AppState {
    fn redis(&self) -> &ConnectionManager { &self.redis }
}

// ✨ Magic: AppState now automatically implements:
// - ProjectState (because it has HasDatabase + HasRedis)
// - CarState (because it has HasMongoDB + HasRedis)
// - BikeState (because it has HasSqlxPool + HasRedis)
// - Any future API that combines these traits!
```

## Real-World Examples

### Example 1: Project API (PostgreSQL + Redis)

```rust
// libs/apis/project/src/state.rs
pub trait ProjectState: HasDatabase + HasRedis {}
impl<T> ProjectState for T where T: HasDatabase + HasRedis {}

// libs/apis/project/src/controller.rs
pub async fn create_project<S: ProjectState>(
    State(state): State<S>,
    ValidatedJson(body): ValidatedJson<CreateProject>,
) -> Result<Json<ProjectResponse>, AppError> {
    // Access PostgreSQL
    let db = state.db();
    // Access Redis
    let redis = state.redis();
    // ... implementation
}
```

### Example 2: Car API (MongoDB + Redis)

```rust
// libs/apis/car/src/state.rs
pub trait CarState: HasMongoDB + HasRedis {}
impl<T> CarState for T where T: HasMongoDB + HasRedis {}

// libs/apis/car/src/controller.rs
pub async fn create_car<S: CarState>(
    State(state): State<S>,
    ValidatedJson(body): ValidatedJson<CreateCar>,
) -> Result<(StatusCode, Json<CarResponse>), AppError> {
    // Access MongoDB
    let collection = state.mongo().collection::<Car>("cars");
    // Access Redis
    let redis = state.redis();
    // ... implementation
}
```

### Example 3: Bike API (sqlx + Redis)

```rust
// libs/apis/bike/src/state.rs
pub trait BikeState: HasSqlxPool + HasRedis {}
impl<T> BikeState for T where T: HasSqlxPool + HasRedis {}

// libs/apis/bike/src/controller.rs
pub async fn create_bike<S: BikeState>(
    State(state): State<S>,
    ValidatedJson(body): ValidatedJson<CreateBike>,
) -> Result<(StatusCode, Json<BikeResponse>), AppError> {
    // Access PostgreSQL via sqlx
    let pool = state.sqlx_pool();
    let bike = sqlx::query_as::<_, Bike>(
        "INSERT INTO bikes (...) VALUES (...) RETURNING *"
    )
    .bind(...)
    .fetch_one(pool)
    .await?;
    // ... implementation
}
```

### Example 4: Adding a New Analytics API (Zero Modifications!)

```rust
// libs/apis/analytics/src/state.rs - NEW FILE
pub trait AnalyticsState: HasMongoDB + HasRedis {}
impl<T> AnalyticsState for T where T: HasMongoDB + HasRedis {}

// AppState in apps/zerg/api/src/state.rs - NO CHANGES NEEDED!
// It already implements HasMongoDB + HasRedis, so it automatically implements AnalyticsState!
```

## Testing with Composition Pattern

### Creating Test States

```rust
// Minimal test state for Car API
#[derive(Clone)]
struct TestCarState {
    mongo: Database,
    redis: ConnectionManager,
}

impl HasMongoDB for TestCarState {
    fn mongo(&self) -> &Database { &self.mongo }
}

impl HasRedis for TestCarState {
    fn redis(&self) -> &ConnectionManager { &self.redis }
}

// ✨ TestCarState automatically implements CarState!
// No need to implement CarState explicitly
```

### Using Testcontainers

```rust
#[tokio::test]
async fn test_car_api() {
    // Start containers
    let mongo_container = Mongo::default().start().await.unwrap();
    let redis_container = Redis::default().start().await.unwrap();

    // Get connection info
    let mongo_port = mongo_container.get_host_port_ipv4(27017).await.unwrap();
    let redis_port = redis_container.get_host_port_ipv4(6379).await.unwrap();

    // Connect and create state
    let mongo = connect_mongo(mongo_port).await;
    let redis = connect_redis(redis_port).await;

    let state = TestCarState { mongo, redis };

    // Use state with any function requiring CarState
    // Containers automatically cleaned up when test ends
}
```

## Benefits Summary

| Aspect | Without Composition | With Composition |
|--------|-------------------|------------------|
| **Adding new API** | Must add trait impl to AppState | Zero changes to AppState |
| **Code per API** | ~15-20 lines | 2 lines |
| **Coupling** | Tight (AppState knows all APIs) | Loose (APIs independent) |
| **Testing** | Hard (need full AppState) | Easy (minimal test states) |
| **Compile-time safety** | Limited | Full type checking |
| **Flexibility** | APIs must use same state | Mix and match traits |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      Application State                       │
│                                                              │
│  AppState implements:                                        │
│    ✓ HasDatabase (PostgreSQL)                              │
│    ✓ HasMongoDB (MongoDB)                                   │
│    ✓ HasRedis (Redis)                                       │
└──────────────────────┬──────────────────────────────────────┘
                       │
           ┌───────────┴───────────┐
           │                       │
┌──────────▼──────────┐ ┌─────────▼──────────┐
│   Project API       │ │    Car API         │
│                     │ │                    │
│ ProjectState:       │ │ CarState:          │
│  • HasDatabase  ✓   │ │  • HasMongoDB  ✓   │
│  • HasRedis     ✓   │ │  • HasRedis    ✓   │
└─────────────────────┘ └────────────────────┘

✨ Both automatically satisfied by AppState!
```

## Common Patterns

### Pattern 1: API Controller

```rust
pub async fn handler<S: YourApiState>(
    State(state): State<S>,
    // ... other extractors
) -> Result<Json<Response>, AppError> {
    // Use state methods defined by composition traits
    let db = state.db();        // if HasDatabase
    let mongo = state.mongo();  // if HasMongoDB
    let redis = state.redis();  // if HasRedis

    // ... business logic
}
```

### Pattern 2: API Router

```rust
pub fn router<S: YourApiState>() -> Router<S> {
    Router::new()
        .route("/resource", get(list::<S>).post(create::<S>))
        .route("/resource/{id}", get(get_one::<S>).put(update::<S>).delete(delete::<S>))
}
```

### Pattern 3: Integrating into Main API

```rust
// apps/zerg/api/src/api/mod.rs
pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(project_router::<AppState>())
        .merge(car_router::<AppState>())
        // Add more APIs - they all work with AppState!
}
```

## Test Organization

```
tests/
├── Unit Tests (libs/apis/*/src/)
│   ├── model_test.rs      - Model validation, conversions
│   └── state_test.rs      - Composition pattern verification
│
├── Integration Tests (apps/zerg/api/tests/)
│   ├── car_api_test.rs           - Full CRUD operations
│   └── composition_pattern_test.rs - Pattern demonstrations
│
└── All use Testcontainers for self-contained testing
```

## Debugging Tips

### Check Trait Implementation

```rust
// Compile-time check: Does my state implement the trait?
fn check_state_implements_trait<S: CarState>(_state: S) {
    // If this compiles, your state implements CarState
}
```

### Verify Composition

```rust
// Runtime check: Print what traits are available
fn verify_composition<S>()
where
    S: HasMongoDB + HasRedis + Clone + Send + Sync + 'static
{
    println!("State has MongoDB and Redis support!");
}
```

## Migration Checklist

If migrating from old pattern to composition pattern:

- [ ] Create base traits in `libs/app/src/state.rs`
- [ ] Update API state definitions to use composition (2 lines each)
- [ ] Implement base traits on AppState (once)
- [ ] Remove old manual trait implementations
- [ ] Update tests to use testcontainers
- [ ] Verify all tests pass

## Resources

- Full diagrams: See `test-architecture.md`
- Test examples: `apps/zerg/api/tests/`
- API examples: `libs/apis/{project,car}/`
- State definitions: `libs/app/src/state.rs`

## Questions?

**Q: Do I need to modify AppState when adding a new API?**
A: No! If your API uses existing base traits (HasDatabase, HasMongoDB, HasRedis), AppState already implements them.

**Q: What if my new API needs a different database?**
A: Add a new base trait (e.g., `HasElasticsearch`), implement it once on AppState, then compose it in your API trait.

**Q: How does the blanket impl work?**
A: `impl<T> ApiState for T where T: TraitA + TraitB` means "any type T that has TraitA and TraitB automatically gets ApiState". Rust's type system handles this at compile time with zero runtime cost.

**Q: Can I test APIs independently?**
A: Yes! Create a minimal test state that only implements the traits your API needs. No need for full AppState in unit tests.
