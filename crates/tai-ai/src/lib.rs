mod config;
mod provider;

pub use config::{
    load_active_model, load_providers, resolve_active, save_active_model, ActiveModel,
    ProviderConfig,
};

use provider::{get_client, AiClient};

use futures::StreamExt;
use rig::{
    agent::MultiTurnStreamItem,
    client::CompletionClient,
    completion::Prompt,
    streaming::{StreamedAssistantContent, StreamingChat},
};
use std::error::Error;

#[derive(Debug, Clone)]
pub enum StreamChunk {
    Reasoning(String),
    Answer(String),
}

pub async fn chat(
    provider: &ProviderConfig,
    model: &str,
    prompt: &str,
) -> Result<String, Box<dyn Error>> {
    let client = get_client(provider);
    let response = match client {
        AiClient::OpenAI(c) => c.agent(model).build().prompt(prompt).await?,
        AiClient::DeepSeek(c) => c.agent(model).build().prompt(prompt).await?,
    };
    Ok(response)
}

pub async fn chat_stream<F>(
    provider: &ProviderConfig,
    model: &str,
    prompt: &str,
    mut on_chunk: F,
) -> Result<String, Box<dyn Error>>
where
    F: FnMut(StreamChunk) -> Result<(), Box<dyn Error>>,
{
    let client = get_client(provider);
    let mut full_response = String::new();

    match client {
        AiClient::OpenAI(c) => {
            let agent = c.agent(model).build();
            let mut stream = agent.stream_chat(prompt, Vec::new()).await;

            while let Some(result) = stream.next().await {
                match result {
                    Ok(MultiTurnStreamItem::StreamAssistantItem(content)) => match content {
                        StreamedAssistantContent::Text(text) => {
                            on_chunk(StreamChunk::Answer(text.text.clone()))?;
                            full_response.push_str(&text.text);
                        }
                        StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
                            on_chunk(StreamChunk::Reasoning(reasoning.clone()))?;
                        }
                        _ => {}
                    },
                    Ok(MultiTurnStreamItem::FinalResponse(_)) => break,
                    Ok(_) => {}
                    Err(e) => return Err(format!("Streaming error: {e}").into()),
                }
            }
        }
        AiClient::DeepSeek(c) => {
            let agent = c.agent(model).build();
            let mut stream = agent.stream_chat(prompt, Vec::new()).await;

            while let Some(result) = stream.next().await {
                match result {
                    Ok(MultiTurnStreamItem::StreamAssistantItem(content)) => match content {
                        StreamedAssistantContent::Text(text) => {
                            on_chunk(StreamChunk::Answer(text.text.clone()))?;
                            full_response.push_str(&text.text);
                        }
                        StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
                            on_chunk(StreamChunk::Reasoning(reasoning.clone()))?;
                        }
                        _ => {}
                    },
                    Ok(MultiTurnStreamItem::FinalResponse(_)) => break,
                    Ok(_) => {}
                    Err(e) => return Err(format!("Streaming error: {e}").into()),
                }
            }
        }
    }

    Ok(full_response)
}
