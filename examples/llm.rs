use blockless_sdk::*;

/// This example demonstrates how to use the Blockless SDK to interact with two different LLM models.
///
/// It sets up two instances of the BlocklessLlm struct.
/// Each model is configured with a system message that changes the assistant's name.
/// The example then sends chat requests to both models and prints their responses,
/// demonstrating how the same instance maintains state between requests.
fn main() {
    // large model
    let mut llm = BlocklessLlm::new(SupportedModels::Mistral7BInstructV03(None)).unwrap();

    // small model
    let mut llm_small = BlocklessLlm::new(SupportedModels::Llama321BInstruct(None)).unwrap();

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
    println!("llm Response: {}", response);

    let prompt_smol = r#"
    You are a helpful assistant.
    First time I ask, you name will be daisy.
    Second time I ask, you name will be hector.
    "#;
    llm_small
        .set_options(LlmOptions {
            system_message: prompt_smol.to_string(),
            top_p: Some(0.5),
            ..Default::default()
        })
        .unwrap();

    let response = llm_small.chat_request("What is your name?").unwrap();
    println!("llm_small Response: {}", response);

    let response = llm_small.chat_request("What is your name?").unwrap();
    println!("llm_small Response: {}", response);

    // test if same instance is used in host/runtime
    let response = llm.chat_request("What is your name?").unwrap();
    println!("llm Response: {}", response);
}
