use blockless_sdk::*;

/// This example demonstrates how to use the Blockless SDK to interact with two different LLM models
/// and use MCP to call the tools.

fn main() {
    // large model
    let mut llm = BlocklessLlm::new(Models::Custom(
    ))
    .unwrap();

    // Assume we have two tools running on different ports
    // 1. http://localhost:3001/sse - add
    // 2. http://localhost:3002/sse - multiply
    llm.set_options(LlmOptions::default().with_tools_sse_urls(vec![
        "http://localhost:3001/sse".to_string(),
        "http://localhost:3002/sse".to_string(),
    ]))
    .unwrap();

    let response = llm
        .chat_request("Add the following numbers: 1215, 2213")
        .unwrap();
    println!("llm Response: {}", response);

    let response = llm.chat_request("Multiply 1215 by 2213").unwrap();
    println!("llm Response: {}", response);
}
