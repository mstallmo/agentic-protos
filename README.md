# Agentic-Protos: gRPC Server with SQLite

> [!IMPORTANT]
> All code in this repository has been generated using the Agentic Editing feature of the Zed code editor with the Claude 3.7 thinking model.
> This repository is an experiment in using the Agentic Editing feature to completely build out a project. The intent is to explore the capabilities
> of the feature and leveraging LLMs for this kind of workflow in general.

A Rust implementation of a gRPC server and client using [Tonic](https://github.com/hyperium/tonic) with SQLite persistence.

## Overview

This project demonstrates how to:

- Define and compile Protocol Buffer service definitions
- Implement a gRPC server using Tonic
- Create a gRPC client to communicate with the server
- Maintain persistent state using SQLite database
- Apply Test-Driven Development (TDD) principles
- Structure a Rust project with gRPC and database components
- Use SQL migrations to manage database schema changes

## Project Structure

```
agentic-protos/
├── Cargo.toml           # Project dependencies
├── build.rs             # Build script for compiling protobufs
├── data.db              # SQLite database file (created at runtime)
├── migrations/          # SQL migration files
│   ├── 20240516000000_create_counters_table.sql
│   └── 20240516000001_add_counter_stats.sql
├── protos/              # Protocol Buffer definitions
│   └── hello_service.proto
├── src/
│   ├── main.rs          # Server implementation
│   ├── database.rs      # SQLite database operations
│   ├── tdd_sample.rs    # TDD example module
│   └── bin/
│       └── client.rs    # Client implementation
└── test_grpc.sh         # Test script to run both server and client
```

## Service Definition

The service is defined in `protos/hello_service.proto`:

```protobuf
syntax = "proto3";
package hello_service;

service HelloService {
  // Basic greeting service
  rpc SayHello(HelloRequest) returns (HelloResponse) {}

  // Counter management methods with SQLite persistence
  rpc IncrementCounter(IncrementCounterRequest) returns (IncrementCounterResponse) {}
  rpc GetCounter(GetCounterRequest) returns (GetCounterResponse) {}
}

// Message definitions for greeting service
message HelloRequest {
  string name = 1;
}

message HelloResponse {
  string message = 1;
}

// Message definitions for counter service
message IncrementCounterRequest {
  int32 increment_by = 1;
}

message IncrementCounterResponse {
  int32 value = 1;
}

message GetCounterRequest {
  // Empty request
}

message GetCounterResponse {
  int32 value = 1;
}
```

## SQLite Integration

The project uses SQLite via the `sqlx` crate to provide persistent storage for counters:

- `database.rs` implements a clean interface for database operations
- Counter values are stored in a SQLite database (`data.db`)
- Data persists between server restarts
- Transactions ensure data integrity during concurrent operations
- SQL migrations automatically apply schema changes on startup
- Statistics tracking for counter operations

## Service Implementation

The server implements three RPC methods:

1. **SayHello** - Takes a name and returns a personalized greeting
2. **IncrementCounter** - Increments a counter in the SQLite database and returns its new value
3. **GetCounter** - Returns the current value of the counter from the SQLite database

The counter is persisted in SQLite, making it survive server restarts.

## Test-Driven Development Sample

The project includes a simple TDD example in `src/tdd_sample.rs`:

- `add_two` function that adds two integers together
- Comprehensive test cases covering various scenarios

To run the tests:

```bash
cargo test
```

## Prerequisites

- Rust (latest stable version recommended)
- Protocol Buffers compiler (`protoc`)
- SQLite (usually included with the `sqlx` crate)

## Building

```bash
cargo build
```

## Running

### Server

To start the gRPC server:

```bash
cargo run
```

The server will:
1. Create or connect to the SQLite database (`data.db`)
2. Apply any pending SQL migrations from the `migrations` directory
3. Initialize the database schema if needed
4. Listen on `[::1]:50052` (IPv6 localhost, port 50052)

### Client

To run the gRPC client (while the server is running):

```bash
cargo run --bin client
```

The client will:
1. Call `SayHello` to get a greeting
2. Call `GetCounter` to check the current counter value from the database
3. Call `IncrementCounter` with various values to update the counter
4. Call `GetCounter` again to see the updated counter value

Since the counter is stored in SQLite, its value persists between server restarts. Each time you run the client, the counter will continue to increment from its previous value.

### Combined Test

To test both server and client together:

```bash
./test_grpc.sh
```

This script:
1. Builds the project
2. Checks for an existing SQLite database
3. Applies any pending migrations
4. Starts the server in the background
5. Runs the client
6. Shuts down the server

## Database Details

The SQLite database (`data.db`) contains a schema that's defined in migration files:

### Initial Schema
```sql
CREATE TABLE IF NOT EXISTS counters (
    id TEXT PRIMARY KEY,
    value INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### Stats Addition
```sql
ALTER TABLE counters ADD COLUMN total_increments INTEGER NOT NULL DEFAULT 0;
ALTER TABLE counters ADD COLUMN average_increment REAL NOT NULL DEFAULT 0.0;
ALTER TABLE counters ADD COLUMN highest_value INTEGER NOT NULL DEFAULT 0;
ALTER TABLE counters ADD COLUMN description TEXT;
```

Multiple counters can be tracked by ID, with the default counter using the ID "main_counter".

## Dependencies

- tonic 0.13.0 - gRPC implementation
- prost 0.13.0 - Protocol Buffers implementation
- sqlx 0.8.0 - Async SQLite client with migrations support
- tokio - Async runtime
- anyhow - Error handling

## License

MIT