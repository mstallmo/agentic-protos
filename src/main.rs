//! # HelloService gRPC Server with SQLite
//! 
//! A gRPC server implementation using Tonic and SQLite that provides:
//! - SayHello: Basic greeting service
//! - IncrementCounter: Increments a counter stored in SQLite
//! - GetCounter: Retrieves the current counter value from SQLite
//! - GetCounterStats: Retrieves statistics about the counter

use std::sync::Arc;
use std::net::SocketAddr;
use anyhow::Result;
use tonic::{transport::Server, Request, Response, Status};

// Import our modules
pub mod tdd_sample;
pub mod database;

// Import the database module types
use database::{Database, MAIN_COUNTER_ID};

// Import the generated protobuf code
pub mod hello_service {
    tonic::include_proto!("hello_service");
}

// Import the gRPC service and message types
use hello_service::{
    hello_service_server::{HelloService, HelloServiceServer},
    HelloRequest, HelloResponse,
    IncrementCounterRequest, IncrementCounterResponse,
    GetCounterRequest, GetCounterResponse,
};

/// Implementation of the HelloService gRPC service with SQLite backend
pub struct HelloServiceImpl {
    /// Database connection for persistent storage
    db: Arc<Database>,
}

impl HelloServiceImpl {
    /// Create a new service instance with a database connection
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl HelloService for HelloServiceImpl {
    /// Handles the SayHello RPC method
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let name = request.into_inner().name;
        println!("Got a greeting request from: {}", name);

        let reply = HelloResponse {
            message: format!("Hello {}!", name),
        };

        Ok(Response::new(reply))
    }

    /// Handles the IncrementCounter RPC method
    async fn increment_counter(
        &self,
        request: Request<IncrementCounterRequest>,
    ) -> Result<Response<IncrementCounterResponse>, Status> {
        let increment_by = request.into_inner().increment_by;
        println!("Incrementing counter by: {}", increment_by);
        
        // Increment the counter in the database
        let new_value = self.db.increment_counter(MAIN_COUNTER_ID, increment_by)
            .await
            .map_err(|e| {
                eprintln!("Database error: {:?}", e);
                Status::internal(format!("Database error: {}", e))
            })?;
        
        println!("Counter incremented, new value: {}", new_value);

        // Fetch counter stats if available
        if let Ok(Some((_, total_increments, avg_increment, highest))) = 
            self.db.get_counter_stats(MAIN_COUNTER_ID).await {
            println!(
                "Counter stats: increments={}, avg={:.2}, highest={}", 
                total_increments, avg_increment, highest
            );
        }

        Ok(Response::new(IncrementCounterResponse { value: new_value }))
    }

    /// Handles the GetCounter RPC method
    async fn get_counter(
        &self,
        _request: Request<GetCounterRequest>,  // Fixed unused variable warning with underscore
    ) -> Result<Response<GetCounterResponse>, Status> {
        println!("Getting counter value");
        
        // Get the counter from the database
        let value = self.db.get_counter(MAIN_COUNTER_ID)
            .await
            .map_err(|e| {
                eprintln!("Database error: {:?}", e);
                Status::internal(format!("Database error: {}", e))
            })?;
        
        println!("Current counter value: {}", value);

        Ok(Response::new(GetCounterResponse { value }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize console logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Connect to SQLite database
    println!("Connecting to SQLite database...");
    let db = Database::connect("sqlite:data.db").await?;

    // List all existing counters
    match db.list_counters().await {
        Ok(counters) if !counters.is_empty() => {
            println!("Found {} existing counters:", counters.len());
            for (id, value) in counters {
                println!("  - {}: {}", id, value);
            }
        }
        Ok(_) => println!("No existing counters found"),
        Err(e) => eprintln!("Failed to list counters: {}", e),
    }
    
    // Server address
    let addr: SocketAddr = "[::1]:50052".parse()?;
    
    // Create the service with the database
    let service = HelloServiceImpl::new(Arc::new(db));

    println!("HelloService gRPC server starting on {}", addr);

    // Start the server
    Server::builder()
        .add_service(HelloServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}