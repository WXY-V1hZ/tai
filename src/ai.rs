use rig::{client::CompletionClient, completion::Prompt, providers::openai};
use std::error::Error;

const BASE_URL: &str = "https://api.openai-proxy.org/v1";
const API_KEY: &str = "sk-LKH3Rtb6LwTJuqn36JIDQCb5ZNzMZbR2n6wdt9owYBBh3FT5";
const MODEL_NAME: &str = "gpt-4o-mini";

pub fn create_client() -> Result<openai::Client, Box<dyn Error>> {
    let client = openai::Client::builder()
        .base_url(BASE_URL)
        .api_key(API_KEY)
        .build()?;
    Ok(client)
}

pub async fn chat(prompt: &str) -> Result<String, Box<dyn Error>> {
    let openai_client = create_client()?;

    let gpt4 = openai_client.agent(MODEL_NAME).build();

    let response = gpt4
        .prompt(prompt)
        .await
        .expect("Failed to prompt ai");

    Ok(response)
}
