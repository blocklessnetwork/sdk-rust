use blockless_sdk::rpc::RpcClient;
use serde::{Deserialize, Serialize};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RpcClient::new();

    // Example 1: Simple ping
    println!("=== Example 1: Simple Ping ===");
    match client.ping() {
        Ok(response) => println!("Ping response: {}", response),
        Err(e) => println!("Ping error: {}", e),
    }

    // Example 2: Echo with different data types
    println!("\n=== Example 2: Echo Examples ===");

    // Echo string
    match client.echo("Hello, World!".to_string()) {
        Ok(response) => println!("Echo string: {}", response),
        Err(e) => println!("Echo error: {}", e),
    }

    // Echo number
    match client.echo(42) {
        Ok(response) => println!("Echo number: {}", response),
        Err(e) => println!("Echo error: {}", e),
    }

    // Echo complex object
    #[derive(Serialize, Deserialize, Debug)]
    struct Person {
        name: String,
        age: u32,
    }

    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    match client.echo(person) {
        Ok(response) => println!("Echo person: {:?}", response),
        Err(e) => println!("Echo error: {}", e),
    }

    // Example 3: Get version
    println!("\n=== Example 3: Get Version ===");
    match client.version() {
        Ok(version) => {
            println!("Version info:");
            for (key, value) in version {
                println!("  {}: {}", key, value);
            }
        }
        Err(e) => println!("Version error: {}", e),
    }

    // Example 4: Generic call with custom types
    println!("\n=== Example 4: Generic Call ===");

    #[derive(Serialize, Deserialize, Debug)]
    struct CustomRequest {
        message: String,
        count: u32,
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct CustomResponse {
        processed: String,
        timestamp: u64,
    }

    let request = CustomRequest {
        message: "Test message".to_string(),
        count: 5,
    };

    // This would fail since "custom.process" doesn't exist in our test implementation
    match client.call::<CustomRequest, CustomResponse>("custom.process", Some(request)) {
        Ok(response) => {
            if let Some(result) = response.result {
                println!("Custom response: {:?}", result);
            } else if let Some(error) = response.error {
                println!("Custom error: {} (code: {})", error.message, error.code);
            }
        }
        Err(e) => println!("Custom call error: {}", e),
    }

    // Example 5: Error handling
    println!("\n=== Example 5: Error Handling ===");

    // Try calling a non-existent method
    match client.call::<(), String>("nonexistent.method", None) {
        Ok(response) => {
            if let Some(error) = response.error {
                println!("Expected error: {} (code: {})", error.message, error.code);
            }
        }
        Err(e) => println!("Call error: {}", e),
    }

    println!("\nAll examples completed!");
    Ok(())
}
