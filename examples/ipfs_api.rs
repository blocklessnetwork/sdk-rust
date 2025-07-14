use blockless_sdk::http::{post, HttpClient, MultipartField};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IPFS RPC API Demo - HTTP v2 Client");
    println!("=================================");
    println!("Make sure your IPFS node is running on localhost:5001");
    println!("Docker command: docker run --rm -it --name ipfs_host -p 4001:4001 -p 4001:4001/udp -p 8080:8080 -p 5001:5001 ipfs/kubo");
    println!("Note: If you get CORS errors, configure CORS with:");
    println!("  docker exec ipfs_host ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '[\"*\"]'");
    println!("  docker exec ipfs_host ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '[\"GET\", \"POST\", \"PUT\", \"DELETE\", \"OPTIONS\"]'");
    println!("  docker restart ipfs_host\n");

    // Create HTTP client configured for IPFS RPC API
    let client = HttpClient::builder()
        .timeout(30000) // 30 seconds for file operations
        .build();

    let ipfs_api_base = "http://localhost:5001/api/v0";

    // Example 1: Simple POST request - Get node version
    println!("1. GET node version (POST with no body):");
    match client.post(format!("{}/version", ipfs_api_base)).send() {
        Ok(response) => {
            println!("   Status: {}", response.status());
            if response.is_success() {
                if let Ok(version_info) = response.json::<serde_json::Value>() {
                    println!(
                        "   IPFS Version: {}",
                        version_info
                            .get("Version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                    );
                    println!(
                        "   Commit: {}",
                        version_info
                            .get("Commit")
                            .and_then(|c| c.as_str())
                            .unwrap_or("unknown")
                    );
                }
            }
        }
        Err(e) => println!("   Error: {} (Is IPFS running?)", e),
    }

    // Example 2: POST with query parameters - Get node ID
    println!("\n2. GET node ID (POST with query parameters):");
    match client
        .post(format!("{}/id", ipfs_api_base))
        .query("format", "json")
        .send()
    {
        Ok(response) => {
            println!("   Status: {}", response.status());
            if response.is_success() {
                if let Ok(id_info) = response.json::<serde_json::Value>() {
                    println!(
                        "   Node ID: {}",
                        id_info
                            .get("ID")
                            .and_then(|id| id.as_str())
                            .unwrap_or("unknown")
                    );
                    if let Some(addresses) = id_info.get("Addresses") {
                        if let Some(addr_array) = addresses.as_array() {
                            if !addr_array.is_empty() {
                                println!("   First Address: {}", addr_array[0]);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 3: POST with multipart file upload - Add file to IPFS
    println!("\n3. ADD file to IPFS (POST with multipart upload):");
    let file_content = b"Hello from Blockless SDK HTTP v2 client!\nThis file was uploaded to IPFS using multipart form data.";
    let multipart_fields = vec![MultipartField::file(
        "file", // IPFS expects 'file' as the field name
        file_content.to_vec(),
        "hello-blockless.txt",
        Some("text/plain".to_string()),
    )];

    match client
        .post(format!("{}/add", ipfs_api_base))
        .query("pin", "true") // Pin the file after adding
        .multipart(multipart_fields)
        .send()
    {
        Ok(response) => {
            println!("   Status: {}", response.status());
            if response.is_success() {
                if let Ok(add_result) = response.json::<serde_json::Value>() {
                    let hash = add_result
                        .get("Hash")
                        .and_then(|h| h.as_str())
                        .unwrap_or("unknown");
                    let name = add_result
                        .get("Name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown");
                    let size = add_result
                        .get("Size")
                        .and_then(|s| s.as_str())
                        .unwrap_or("0");
                    println!("   Added file: {}", name);
                    println!("   IPFS Hash: {}", hash);
                    println!("   Size: {} bytes", size);

                    // Store hash for later examples
                    if hash != "unknown" {
                        demonstrate_file_operations(&client, ipfs_api_base, hash)?;
                    }
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 4: Repository stats (POST with query parameters)
    println!("\n4. GET repository statistics (POST with boolean parameters):");
    match client
        .post(format!("{}/repo/stat", ipfs_api_base))
        .query("human", "true")
        .send()
    {
        Ok(response) => {
            println!("   Status: {}", response.status());
            if response.is_success() {
                if let Ok(repo_stats) = response.json::<serde_json::Value>() {
                    println!(
                        "   Repo Size: {}",
                        repo_stats
                            .get("RepoSize")
                            .unwrap_or(&serde_json::Value::Number(0.into()))
                    );
                    println!(
                        "   Storage Max: {}",
                        repo_stats
                            .get("StorageMax")
                            .unwrap_or(&serde_json::Value::Number(0.into()))
                    );
                    println!(
                        "   Num Objects: {}",
                        repo_stats
                            .get("NumObjects")
                            .unwrap_or(&serde_json::Value::Number(0.into()))
                    );
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 5: Pin operations - List pinned objects
    println!("\n5. LIST pinned objects (POST with type filter):");
    match client
        .post(format!("{}/pin/ls", ipfs_api_base))
        .query("type", "recursive")
        .query("stream", "true")
        .send()
    {
        Ok(response) => {
            println!("   Status: {}", response.status());
            if response.is_success() {
                if let Ok(pin_list) = response.json::<serde_json::Value>() {
                    if let Some(keys) = pin_list.get("Keys").and_then(|k| k.as_object()) {
                        println!("   Pinned objects count: {}", keys.len());
                        // Show first few pinned objects
                        for (hash, info) in keys.iter().take(3) {
                            if let Some(pin_type) = info.get("Type") {
                                println!("   - {} ({})", hash, pin_type);
                            }
                        }
                        if keys.len() > 3 {
                            println!("   ... and {} more", keys.len() - 3);
                        }
                    }
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    // Example 6: Module-level convenience function
    println!("\n6. GET swarm peers (using module-level function):");
    match post(format!("{}/swarm/peers", ipfs_api_base))
        .query("verbose", "false")
        .send()
    {
        Ok(response) => {
            println!("   Status: {}", response.status());
            if response.is_success() {
                if let Ok(peers_info) = response.json::<serde_json::Value>() {
                    if let Some(peers) = peers_info.get("Peers").and_then(|p| p.as_array()) {
                        println!("   Connected peers: {}", peers.len());
                        // Show first few peers
                        for peer in peers.iter().take(2) {
                            if let Some(peer_id) = peer.get("Peer") {
                                if let Some(addr) = peer.get("Addr") {
                                    println!(
                                        "   - Peer: {}...{}",
                                        &peer_id.as_str().unwrap_or("")[..8],
                                        &peer_id.as_str().unwrap_or("")[peer_id
                                            .as_str()
                                            .unwrap_or("")
                                            .len()
                                            .saturating_sub(8)..]
                                    );
                                    println!("     Address: {}", addr);
                                }
                            }
                        }
                        if peers.len() > 2 {
                            println!("   ... and {} more peers", peers.len() - 2);
                        }
                    }
                }
            }
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n✅ IPFS API Demo completed!");
    println!("This example demonstrated:");
    println!("  • POST requests with no body (version, id)");
    println!("  • POST with query parameters (repo/stat, pin/ls)");
    println!("  • POST with multipart file upload (add)");
    println!("  • POST with binary responses (cat - in demonstrate_file_operations)");
    println!("  • Module-level convenience functions (swarm/peers)");
    println!("  • Different response types (JSON, binary)");

    Ok(())
}

/// Demonstrates file operations with the uploaded file
fn demonstrate_file_operations(
    client: &HttpClient,
    api_base: &str,
    file_hash: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Example: Get file content (binary response)
    println!("\n   📄 GET file content (POST returning binary data):");
    match client
        .post(format!("{}/cat", api_base))
        .query("arg", file_hash)
        .send()
    {
        Ok(response) => {
            println!("      Status: {}", response.status());
            if response.is_success() {
                match response.text() {
                    Ok(content) => {
                        println!(
                            "      File content: {}",
                            content.lines().next().unwrap_or("empty")
                        );
                        println!("      Content length: {} bytes", content.len());
                    }
                    Err(_) => {
                        println!("      Binary content: {} bytes", response.bytes().len());
                    }
                }
            }
        }
        Err(e) => println!("      Error: {}", e),
    }

    // Example: Pin the file explicitly (idempotent operation)
    println!("\n   📌 PIN file (POST with path parameter):");
    match client
        .post(format!("{}/pin/add", api_base))
        .query("arg", file_hash)
        .query("recursive", "false")
        .send()
    {
        Ok(response) => {
            println!("      Status: {}", response.status());
            if response.is_success() {
                if let Ok(pin_result) = response.json::<serde_json::Value>() {
                    if let Some(pins) = pin_result.get("Pins").and_then(|p| p.as_array()) {
                        println!("      Pinned {} objects", pins.len());
                        for pin in pins {
                            println!("      - {}", pin.as_str().unwrap_or("unknown"));
                        }
                    }
                }
            }
        }
        Err(e) => println!("      Error: {}", e),
    }

    Ok(())
}
