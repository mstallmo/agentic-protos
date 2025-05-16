//! # HelloService gRPC Client
//!
//! A client for testing the HelloService gRPC server with SQLite migrations.
//! This client connects to the server and tests all available RPC methods.

use anyhow::Result;
use hello_service::hello_service_client::HelloServiceClient;
use hello_service::{HelloRequest, IncrementCounterRequest, GetCounterRequest};
use tokio::time::{sleep, Duration};

// Import the generated protobuf code
pub mod hello_service {
    tonic::include_proto!("hello_service");
}

/// Main function to run the gRPC client.
#[tokio::main]
async fn main() -> Result<()> {
    // Create a channel to the server
    println!("Connecting to gRPC server at [::1]:50052...");
    let channel = tonic::transport::Channel::from_static("http://[::1]:50052")
        .connect()
        .await?;

    // Create a client using the channel
    let mut client = HelloServiceClient::new(channel);
    println!("✅ Connection established successfully");

    // Test 1: SayHello RPC
    println!("\n=== Testing SayHello RPC ===");
    let request = tonic::Request::new(HelloRequest {
        name: "SQLite Migration".into(),
    });

    match client.say_hello(request).await {
        Ok(response) => {
            println!("✅ Greeting received: {}", response.into_inner().message);
        },
        Err(err) => {
            println!("❌ SayHello failed: {}", err);
        }
    }

    // Test 2: Get initial counter value
    println!("\n=== Testing GetCounter RPC (initial value) ===");
    let request = tonic::Request::new(GetCounterRequest {});

    match client.get_counter(request).await {
        Ok(response) => {
            let value = response.into_inner().value;
            println!("✅ Current counter value: {}", value);
            println!("   This value persists between server restarts because it's stored in SQLite");
            println!("   And now includes migrations for statistics tracking!");
        },
        Err(err) => {
            println!("❌ GetCounter failed: {}", err);
        }
    }

    // Test 3: Make multiple increments to test statistics tracking
    println!("\n=== Testing IncrementCounter RPC multiple times ===");
    
    // Increment a few times with different values
    let increments = [1, 2, 5, 10];
    
    for increment in increments {
        let request = tonic::Request::new(IncrementCounterRequest {
            increment_by: increment,
        });

        match client.increment_counter(request).await {
            Ok(response) => {
                println!("✅ Counter incremented by {}, new value: {}", 
                    increment, response.into_inner().value);
                
                // Small delay to make output more readable
                sleep(Duration::from_millis(100)).await;
            },
            Err(err) => {
                println!("❌ IncrementCounter failed: {}", err);
                break;
            }
        }
    }

    // Test 4: Get final counter value
    println!("\n=== Testing GetCounter RPC (final value) ===");
    let request = tonic::Request::new(GetCounterRequest {});

    match client.get_counter(request).await {
        Ok(response) => {
            let value = response.into_inner().value;
            println!("✅ Final counter value: {}", value);
            println!("   This value is stored in a SQLite database file");
            println!("   The database now tracks statistics like total increments and average!");
        },
        Err(err) => {
            println!("❌ GetCounter failed: {}", err);
        }
    }

    println!("\n=== Completed All Tests ===");
    println!("The counter value and statistics are persisted in SQLite");
    println!("Each time you run the test, the counters will continue from where they left off");
    println!("The migrations system ensures database schema stays up to date");

    Ok(())
}