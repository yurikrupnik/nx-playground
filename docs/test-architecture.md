# Test Architecture & Composition Pattern

This document provides visual diagrams explaining the test structure, flows, and the composition pattern used in the Zerg API project.

## 1. Overall Test Structure

```mermaid
graph TB
    subgraph "Test Organization"
        A[Test Suite] --> B[Unit Tests]
        A --> C[Integration Tests]
        A --> D[Pattern Tests]

        B --> B1[apis_project<br/>2 tests]
        B --> B2[apis_car<br/>8 tests]

        C --> C1[car_api_test<br/>6 CRUD tests]

        D --> D1[composition_pattern_test<br/>4 pattern tests]
        D --> D2[state_test in libs<br/>Composition verification]
    end

    subgraph "Test Infrastructure"
        E[Testcontainers] --> F[PostgreSQL<br/>Container]
        E --> G[MongoDB<br/>Container]
        E --> H[Redis<br/>Container]
    end

    B1 -.uses.-> F
    B1 -.uses.-> H
    B2 -.uses.-> G
    B2 -.uses.-> H
    C1 -.uses.-> F
    C1 -.uses.-> G
    C1 -.uses.-> H
    D1 -.uses.-> F
    D1 -.uses.-> G
    D1 -.uses.-> H

    style A fill:#e1f5ff
    style E fill:#fff4e1
    style B1 fill:#e8f5e9
    style B2 fill:#e8f5e9
    style C1 fill:#fff3e0
    style D1 fill:#f3e5f5
```

## 2. Testcontainers Lifecycle Flow

```mermaid
sequenceDiagram
    participant Test as Test Function
    participant TC as Testcontainers
    participant Docker as Docker
    participant DB as Database
    participant App as Application

    Note over Test,App: Test Execution Begins

    Test->>TC: Start containers
    TC->>Docker: Pull & start Postgres image
    TC->>Docker: Pull & start MongoDB image
    TC->>Docker: Pull & start Redis image

    Docker-->>TC: Containers running
    TC-->>Test: Connection info (ports)

    Test->>DB: Connect to databases
    DB-->>Test: Connections established

    Test->>App: Create AppState with connections
    Test->>App: Execute test operations
    App->>DB: CRUD operations
    DB-->>App: Results
    App-->>Test: Assertions

    Note over Test,App: Test Execution Complete

    Test->>TC: Test finished (implicit)
    TC->>Docker: Stop & remove containers
    Docker-->>TC: Cleanup complete

    Note over Test,App: Ready for next test
```

## 3. Composition Pattern: Trait Hierarchy

```mermaid
graph TB
    subgraph "Base Composition Traits"
        HasDB[HasDatabase<br/>PostgreSQL/SeaORM]
        HasMongo[HasMongoDB<br/>MongoDB]
        HasRedis[HasRedis<br/>Redis]
    end

    subgraph "API-Specific Traits"
        ProjectState[ProjectState<br/>HasDatabase + HasRedis]
        CarState[CarState<br/>HasMongoDB + HasRedis]
        FutureAPI[Future API<br/>Any combination]
    end

    subgraph "Application State"
        AppState[AppState<br/>Implements all base traits]
    end

    subgraph "Automatic Implementation via Blanket Impl"
        Blanket["impl&lt;T&gt; ProjectState for T<br/>where T: HasDatabase + HasRedis"]
        Blanket2["impl&lt;T&gt; CarState for T<br/>where T: HasMongoDB + HasRedis"]
    end

    HasDB -.->|"part of"| ProjectState
    HasRedis -.->|"part of"| ProjectState
    HasRedis -.->|"part of"| CarState
    HasMongo -.->|"part of"| CarState

    AppState -->|"implements"| HasDB
    AppState -->|"implements"| HasMongo
    AppState -->|"implements"| HasRedis

    Blanket -.->|"automatically provides"| ProjectState
    Blanket2 -.->|"automatically provides"| CarState

    AppState -->|"✨ auto-implements"| ProjectState
    AppState -->|"✨ auto-implements"| CarState
    AppState -->|"✨ auto-implements"| FutureAPI

    style HasDB fill:#e3f2fd
    style HasMongo fill:#e8f5e9
    style HasRedis fill:#fff3e0
    style ProjectState fill:#f3e5f5
    style CarState fill:#fce4ec
    style AppState fill:#ffebee
    style Blanket fill:#fff9c4
    style Blanket2 fill:#fff9c4
```

## 4. API Request Flow Through Layers

```mermaid
graph LR
    subgraph "HTTP Layer"
        A[HTTP Request<br/>POST /car]
    end

    subgraph "Router Layer"
        B[Axum Router<br/>car_router]
    end

    subgraph "Controller Layer"
        C[create_car&lt;S: CarState&gt;<br/>Validates request]
    end

    subgraph "State Extraction"
        D[State&lt;S&gt; extractor<br/>Provides state]
    end

    subgraph "Business Logic"
        E[Access state.mongo<br/>via trait method]
        F[MongoDB operations<br/>Insert document]
    end

    subgraph "Response"
        G[CarResponse<br/>JSON serialization]
        H[HTTP 201 Created]
    end

    A --> B
    B --> C
    C --> D
    D --> E
    E --> F
    F --> G
    G --> H

    style A fill:#e1f5ff
    style C fill:#fff4e1
    style E fill:#e8f5e9
    style G fill:#f3e5f5
```

## 5. Test Flow: Integration Test Example

```mermaid
sequenceDiagram
    participant Test as test_create_car()
    participant TC as Testcontainers
    participant State as AppState
    participant Router as Axum Router
    participant Controller as create_car
    participant MongoDB as MongoDB

    Note over Test,MongoDB: Setup Phase

    Test->>TC: create_test_state()
    TC->>TC: Start Postgres container
    TC->>TC: Start MongoDB container
    TC->>TC: Start Redis container
    TC-->>Test: TestContext with connections

    Test->>State: Create AppState
    Test->>Router: api::routes().with_state(state)

    Note over Test,MongoDB: Execution Phase

    Test->>Router: POST /car with JSON body
    Router->>Controller: Route to create_car handler
    Controller->>Controller: Validate request body
    Controller->>State: Extract State<S>
    State->>Controller: Provide state
    Controller->>MongoDB: state.mongo().collection()
    Controller->>MongoDB: insert_one(car)
    MongoDB-->>Controller: Insert result
    Controller->>Controller: Convert Car → CarResponse
    Controller-->>Router: Json(CarResponse)
    Router-->>Test: HTTP 201 Created

    Note over Test,MongoDB: Assertion Phase

    Test->>Test: assert_eq!(status, 201)
    Test->>Test: Verify response body

    Note over Test,MongoDB: Cleanup Phase

    Test->>TC: Test ends (automatic)
    TC->>TC: Stop & remove all containers
```

## 6. Composition Pattern Benefits

```mermaid
graph TB
    subgraph "Without Composition Pattern ❌"
        Old1[Every new API requires:<br/>Manual trait impl on AppState]
        Old2[Tight coupling between<br/>AppState and API traits]
        Old3[Repetitive boilerplate<br/>for each API]
        Old4[Hard to test with<br/>mock states]
    end

    subgraph "With Composition Pattern ✅"
        New1[Small, focused traits<br/>HasDatabase, HasMongoDB, HasRedis]
        New2[APIs compose only<br/>what they need]
        New3[Blanket impl provides<br/>automatic satisfaction]
        New4[Zero modification<br/>to AppState for new APIs]
        New5[Easy to create<br/>minimal test states]
    end

    Old1 -.X.-> New1
    Old2 -.X.-> New2
    Old3 -.X.-> New3
    Old4 -.X.-> New5

    style Old1 fill:#ffebee
    style Old2 fill:#ffebee
    style Old3 fill:#ffebee
    style Old4 fill:#ffebee
    style New1 fill:#e8f5e9
    style New2 fill:#e8f5e9
    style New3 fill:#e8f5e9
    style New4 fill:#e8f5e9
    style New5 fill:#e8f5e9
```

## 7. State Trait Composition in Action

```mermaid
graph TB
    subgraph "Step 1: Define Base Traits"
        T1["trait HasDatabase {<br/>  fn db() -> &DatabaseConnection<br/>}"]
        T2["trait HasMongoDB {<br/>  fn mongo() -> &Database<br/>}"]
        T3["trait HasRedis {<br/>  fn redis() -> &ConnectionManager<br/>}"]
    end

    subgraph "Step 2: API Defines Requirements"
        API1["trait ProjectState:<br/>  HasDatabase + HasRedis {}"]
        API2["trait CarState:<br/>  HasMongoDB + HasRedis {}"]
    end

    subgraph "Step 3: Blanket Implementation"
        B1["impl&lt;T&gt; ProjectState for T<br/>where T: HasDatabase + HasRedis"]
        B2["impl&lt;T&gt; CarState for T<br/>where T: HasMongoDB + HasRedis"]
    end

    subgraph "Step 4: AppState Implementation"
        AS["struct AppState {<br/>  db: DatabaseConnection,<br/>  mongo: Database,<br/>  redis: ConnectionManager<br/>}"]
        I1["impl HasDatabase for AppState"]
        I2["impl HasMongoDB for AppState"]
        I3["impl HasRedis for AppState"]
    end

    subgraph "Step 5: Automatic Satisfaction ✨"
        R1["AppState automatically<br/>implements ProjectState!"]
        R2["AppState automatically<br/>implements CarState!"]
    end

    T1 --> API1
    T3 --> API1
    T2 --> API2
    T3 --> API2

    API1 --> B1
    API2 --> B2

    AS --> I1
    AS --> I2
    AS --> I3

    I1 --> R1
    I3 --> R1
    I2 --> R2
    I3 --> R2

    B1 -.->|"enables"| R1
    B2 -.->|"enables"| R2

    style T1 fill:#e3f2fd
    style T2 fill:#e8f5e9
    style T3 fill:#fff3e0
    style API1 fill:#f3e5f5
    style API2 fill:#fce4ec
    style AS fill:#ffebee
    style R1 fill:#c8e6c9
    style R2 fill:#c8e6c9
```

## 8. Test Coverage Matrix

```mermaid
graph TB
    subgraph "Unit Tests - libs/apis/project"
        UP1[test_project_state_composition<br/>✅ PostgreSQL + Redis]
        UP2[test_state_traits_are_composable<br/>✅ Compile-time check]
    end

    subgraph "Unit Tests - libs/apis/car"
        UC1[test_create_car_validation<br/>✅ Model validation]
        UC2[test_car_to_response_conversion<br/>✅ Type conversion]
        UC3[test_update_car_partial_fields<br/>✅ Partial updates]
        UC4[test_car_response_serialization<br/>✅ JSON serialization]
        UC5[test_create_car_validation_fails_*<br/>✅ Validation errors]
        UC6[test_car_state_composition<br/>✅ MongoDB + Redis]
        UC7[test_state_traits_are_composable<br/>✅ Compile-time check]
    end

    subgraph "Integration Tests - apps/zerg/api"
        IT1[test_create_car<br/>✅ POST /car]
        IT2[test_list_cars<br/>✅ GET /cars]
        IT3[test_get_car_not_found<br/>✅ GET /car/:id 404]
        IT4[test_update_car<br/>✅ PUT /car/:id]
        IT5[test_delete_car<br/>✅ DELETE /car/:id]
        IT6[test_create_car_validation_fails<br/>✅ POST /car validation]
    end

    subgraph "Pattern Tests - apps/zerg/api"
        PT1[test_composition_pattern_flexibility<br/>✅ Single state, multiple APIs]
        PT2[test_minimal_state_for_specific_api<br/>✅ Minimal test states]
        PT3[test_composition_provides_compile_time_safety<br/>✅ Type safety]
        PT4[test_no_modification_needed_for_new_apis<br/>✅ Zero modification]
    end

    style UP1 fill:#e8f5e9
    style UP2 fill:#e8f5e9
    style UC1 fill:#e8f5e9
    style UC2 fill:#e8f5e9
    style UC3 fill:#e8f5e9
    style UC4 fill:#e8f5e9
    style UC5 fill:#e8f5e9
    style UC6 fill:#e8f5e9
    style UC7 fill:#e8f5e9
    style IT1 fill:#fff3e0
    style IT2 fill:#fff3e0
    style IT3 fill:#fff3e0
    style IT4 fill:#fff3e0
    style IT5 fill:#fff3e0
    style IT6 fill:#fff3e0
    style PT1 fill:#f3e5f5
    style PT2 fill:#f3e5f5
    style PT3 fill:#f3e5f5
    style PT4 fill:#f3e5f5
```

## 9. Adding a New API: Zero Modification Flow

```mermaid
sequenceDiagram
    participant Dev as Developer
    participant NewAPI as New Analytics API
    participant Traits as Composition Traits
    participant AppState as AppState
    participant System as Type System

    Note over Dev,System: No changes to AppState needed!

    Dev->>NewAPI: Create AnalyticsState trait
    NewAPI->>Traits: Compose: HasMongoDB + HasRedis

    NewAPI->>NewAPI: Define trait AnalyticsState
    Note right of NewAPI: trait AnalyticsState:<br/>  HasMongoDB + HasRedis {}

    NewAPI->>NewAPI: Add blanket impl
    Note right of NewAPI: impl<T> AnalyticsState for T<br/>where T: HasMongoDB + HasRedis

    System->>AppState: Check: implements HasMongoDB? ✅
    System->>AppState: Check: implements HasRedis? ✅
    System->>AppState: ✨ AppState implements AnalyticsState!

    Note over Dev,System: No manual implementation needed!

    Dev->>NewAPI: Create controllers
    NewAPI->>NewAPI: fn handler<S: AnalyticsState>(...)
    Dev->>NewAPI: Create router
    NewAPI->>AppState: Use AppState directly

    Note over Dev,System: API works immediately!
```

## 10. Testcontainers Architecture

```mermaid
graph TB
    subgraph "Test Execution Environment"
        T[Test Function]
    end

    subgraph "Testcontainers Library"
        TC[Testcontainers<br/>Container Management]
        TCM[Testcontainers Modules<br/>Postgres, Mongo, Redis]
    end

    subgraph "Docker Environment"
        D[Docker Daemon]
        subgraph "Running Containers"
            C1[Postgres 16-alpine<br/>Random port]
            C2[MongoDB latest<br/>Random port]
            C3[Redis latest<br/>Random port]
        end
    end

    subgraph "Test Application"
        AS[AppState<br/>with connections]
        API[API Handlers]
    end

    T -->|"1. Start"| TC
    TC --> TCM
    TCM -->|"2. Pull images"| D
    D -->|"3. Create"| C1
    D -->|"3. Create"| C2
    D -->|"3. Create"| C3

    C1 -.->|"4. Get port"| TC
    C2 -.->|"4. Get port"| TC
    C3 -.->|"4. Get port"| TC

    TC -->|"5. Connection info"| T
    T -->|"6. Connect"| C1
    T -->|"6. Connect"| C2
    T -->|"6. Connect"| C3

    T -->|"7. Create state"| AS
    AS -->|"8. Test requests"| API
    API -.->|"9. Database ops"| C1
    API -.->|"9. Database ops"| C2
    API -.->|"9. Database ops"| C3

    T -->|"10. Cleanup"| TC
    TC -->|"11. Stop & remove"| D

    style T fill:#e1f5ff
    style TC fill:#fff4e1
    style C1 fill:#e3f2fd
    style C2 fill:#e8f5e9
    style C3 fill:#fff3e0
    style AS fill:#ffebee
    style API fill:#f3e5f5
```

## Key Takeaways

### Composition Pattern
- **Zero Boilerplate**: Adding new APIs requires zero modifications to AppState
- **Type Safety**: Compile-time verification that state has required dependencies
- **Flexibility**: Mix and match traits based on API needs
- **Testability**: Easy to create minimal mock states for testing

### Testcontainers Benefits
- **Self-Contained**: No external database setup required
- **Isolated**: Each test gets fresh containers
- **Portable**: Works on any machine with Docker
- **CI/CD Ready**: Perfect for automated testing pipelines
- **Real Databases**: Tests run against actual database engines, not mocks

### Test Coverage
- **20 total tests**: All passing with testcontainers
- **3 database types**: PostgreSQL, MongoDB, Redis
- **Multiple layers**: Unit, integration, and pattern tests
- **Complete CRUD**: Full coverage of API operations
- **Pattern verification**: Tests prove the composition pattern works
