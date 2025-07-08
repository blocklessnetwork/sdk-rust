use blockless_sdk::http::{get, post, HttpClient, MultipartField};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("====================================");
    println!("HTTP v2 Client Demo");
    println!("====================================");

    println!("\n1. GET request:");
    match get("https://httpbin.org/get").send() {
        Ok(response) => {
            println!("GET Status: {}", response.status());
            println!("GET Success: {}", response.is_success());
        }
        Err(e) => println!("GET Error: {}", e),
    }

    println!("\n2. POST with JSON:");
    let json_data = serde_json::json!({
        "name": "Blockless SDK",
        "version": "2.0",
        "api_style": "reqwest-like"
    });
    match post("https://httpbin.org/post").json(&json_data)?.send() {
        Ok(response) => {
            println!("POST JSON Status: {}", response.status());
            if let Ok(response_json) = response.json::<serde_json::Value>() {
                if let Some(received_json) = response_json.get("json") {
                    println!("Received JSON: {}", received_json);
                }
            }
        }
        Err(e) => println!("POST JSON Error: {}", e),
    }

    println!("\n3. Client instance with default configuration:");
    let mut default_headers = HashMap::new();
    default_headers.insert("User-Agent".to_string(), "Blockless-SDK/2.0".to_string());
    default_headers.insert("Accept".to_string(), "application/json".to_string());
    let client = HttpClient::builder()
        .default_headers(default_headers)
        .timeout(10000)
        .build();
    match client
        .get("https://httpbin.org/get")
        .query("search", "blockless")
        .query("limit", "10")
        .query("format", "json")
        .send()
    {
        Ok(response) => {
            println!("Client GET Status: {}", response.status());
            if let Ok(json_data) = response.json::<serde_json::Value>() {
                if let Some(args) = json_data.get("args") {
                    println!("Query params: {}", args);
                }
            }
        }
        Err(e) => println!("Client GET Error: {}", e),
    }

    println!("\n4. Authentication examples:");
    match client
        .get("https://httpbin.org/basic-auth/user/pass")
        .basic_auth("user", "pass")
        .send()
    {
        Ok(response) => {
            println!("Basic auth status: {}", response.status());
            if let Ok(json_data) = response.json::<serde_json::Value>() {
                println!("Authenticated: {:?}", json_data.get("authenticated"));
            }
        }
        Err(e) => println!("Basic auth error: {}", e),
    }

    match client
        .get("https://httpbin.org/bearer")
        .bearer_auth("test-token-12345")
        .send()
    {
        Ok(response) => {
            println!("Bearer auth status: {}", response.status());
            if let Ok(json_data) = response.json::<serde_json::Value>() {
                println!("Token received: {:?}", json_data.get("token"));
            }
        }
        Err(e) => println!("Bearer auth error: {}", e),
    }

    println!("\n5. Different request body types:");
    let mut form_data = HashMap::new();
    form_data.insert("name".to_string(), "Blockless".to_string());
    form_data.insert("type".to_string(), "distributed computing".to_string());
    match client
        .post("https://httpbin.org/post")
        .form(form_data)
        .send()
    {
        Ok(response) => {
            println!("Form POST Status: {}", response.status());
            if let Ok(json_data) = response.json::<serde_json::Value>() {
                if let Some(form) = json_data.get("form") {
                    println!("Form data received: {}", form);
                }
            }
        }
        Err(e) => println!("Form POST Error: {}", e),
    }

    println!("\n6. Multipart form with file upload:");
    let multipart_fields = vec![
        MultipartField::text("description", "SDK test file"),
        MultipartField::file(
            "upload",
            b"Hello from Blockless SDK v2!".to_vec(),
            "hello.txt",
            Some("text/plain".to_string()),
        ),
    ];
    match client
        .post("https://httpbin.org/post")
        .multipart(multipart_fields)
        .send()
    {
        Ok(response) => {
            println!("Multipart POST Status: {}", response.status());
            if let Ok(json_data) = response.json::<serde_json::Value>() {
                if let Some(files) = json_data.get("files") {
                    println!("Files uploaded: {}", files);
                }
            }
        }
        Err(e) => println!("Multipart POST Error: {}", e),
    }

    println!("\n7. Binary data:");
    let binary_data = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello" in bytes
    match client
        .post("https://httpbin.org/post")
        .header("Content-Type", "application/octet-stream")
        .body_bytes(binary_data)
        .send()
    {
        Ok(response) => {
            println!("Binary POST Status: {}", response.status());
        }
        Err(e) => println!("Binary POST Error: {}", e),
    }

    println!("\n8. Advanced request building:");
    match client
        .put("https://httpbin.org/put")
        .header("X-Custom-Header", "custom-value")
        .header("X-API-Version", "2.0")
        .query("action", "update")
        .query("id", "12345")
        .timeout(5000)
        .body("Updated data")
        .send()
    {
        Ok(response) => {
            println!("PUT Status: {}", response.status());
            if let Ok(json_data) = response.json::<serde_json::Value>() {
                if let Some(headers) = json_data.get("headers") {
                    println!("Custom headers received: {}", headers);
                }
            }
        }
        Err(e) => println!("PUT Error: {}", e),
    }

    println!("\nDemo completed 🚀");
    Ok(())
}
