use blockless_sdk::*;

/// This example demonstrates how to use the Blockless SDK to interact with two different LLM models.
///
/// It sets up two instances of the BlocklessLlm struct:
/// - One for a large model (Llama-3.1-8B)
/// - One for a small model (SmolLM2-1.7B)
///
/// Each model is configured with a system message that changes the assistant's name.
/// The example then sends chat requests to both models and prints their responses,
/// demonstrating how the same instance maintains state between requests.

fn main() {
    // large model
    let mut llm = BlocklessLlm::new("Llama-3.1-8B-Instruct-q4f32_1-MLC").unwrap();

    // small model
    let mut llm_smol = BlocklessLlm::new("SmolLM2-1.7B-Instruct-q4f16_1-MLC").unwrap();

    let prompt = r#"
    You are a helpful assistant.
    First time I ask, you name will be lucy.
    Second time I ask, you name will be bob.
    "#;
    llm.set_options(LlmOptions {
        system_message: prompt.to_string(),
        top_p: Some(0.5),
        ..Default::default()
    })
    .unwrap();

    let response = llm.chat_request("What is your name?").unwrap();
    println!("LLM Response: {}", response);

    let prompt_smol = r#"
    You are a helpful assistant.
    First time I ask, you name will be daisy.
    Second time I ask, you name will be hector.
    "#;
    llm_smol
        .set_options(LlmOptions {
            system_message: prompt_smol.to_string(),
            top_p: Some(0.5),
            ..Default::default()
        })
        .unwrap();

    let response = llm_smol.chat_request("What is your name?").unwrap();
    println!("LLM Response SmolLM: {}", response);

    let response = llm_smol.chat_request("What is your name?").unwrap();
    println!("LLM Response SmolLM: {}", response);

    // test if same instance is used in host/runtime
    let response = llm.chat_request("What is your name?").unwrap();
    println!("LLM Response: {}", response);

    // For streaming responses, you can use read_response_chunk
    // let mut buf = [0u8; 4096];
    // while let Ok(num) = llm.read_response_chunk(&mut buf) {
    //     if num == 0 {
    //         break;
    //     }
    //     let chunk = String::from_utf8_lossy(&buf[..num as usize]);
    //     println!("Chunk: {}", chunk);
    // }
}
