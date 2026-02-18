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
use tai_core::{TaiError, TaiResult};
use tracing::{debug, error};

#[derive(Debug, Clone)]
pub enum StreamChunk {
    Reasoning(String),
    Answer(String),
}

pub async fn chat(
    provider: &ProviderConfig,
    model: &str,
    prompt: &str,
) -> TaiResult<String> {
    debug!("开始非流式 AI 请求: provider={}, model={}", provider.provider, model);
    debug!("提示词: {}", prompt);
    
    let client = get_client(provider);
    let response = match client {
        AiClient::OpenAI(c) => {
            c.agent(model).build().prompt(prompt).await.map_err(|e| {
                error!("OpenAI API 请求失败: {}", e);
                TaiError::AiError(format!("OpenAI 请求失败: {}", e))
            })?
        }
        AiClient::DeepSeek(c) => {
            c.agent(model).build().prompt(prompt).await.map_err(|e| {
                error!("DeepSeek API 请求失败: {}", e);
                TaiError::AiError(format!("DeepSeek 请求失败: {}", e))
            })?
        }
    };
    
    debug!("AI 请求成功，响应长度: {} 字符", response.len());
    Ok(response)
}

pub async fn chat_stream<F>(
    provider: &ProviderConfig,
    model: &str,
    prompt: &str,
    mut on_chunk: F,
) -> TaiResult<String>
where
    F: FnMut(StreamChunk) -> TaiResult<()>,
{
    debug!("开始流式 AI 请求: provider={}, model={}", provider.provider, model);
    debug!("提示词: {}", prompt);
    
    let client = get_client(provider);
    let mut full_response = String::new();
    let mut chunk_count = 0;

    match client {
        AiClient::OpenAI(c) => {
            let agent = c.agent(model).build();
            let mut stream = agent.stream_chat(prompt, Vec::new()).await;

            while let Some(result) = stream.next().await {
                match result {
                    Ok(MultiTurnStreamItem::StreamAssistantItem(content)) => match content {
                        StreamedAssistantContent::Text(text) => {
                            chunk_count += 1;
                            debug!("收到答案块 #{}: {} 字符", chunk_count, text.text.len());
                            on_chunk(StreamChunk::Answer(text.text.clone()))?;
                            full_response.push_str(&text.text);
                        }
                        StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
                            chunk_count += 1;
                            debug!("收到推理块 #{}: {} 字符", chunk_count, reasoning.len());
                            on_chunk(StreamChunk::Reasoning(reasoning.clone()))?;
                        }
                        _ => {}
                    },
                    Ok(MultiTurnStreamItem::FinalResponse(_)) => {
                        debug!("收到最终响应标记");
                        break;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("流式请求出错: {}", e);
                        return Err(TaiError::AiError(format!("流式请求失败: {}", e)));
                    }
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
                            chunk_count += 1;
                            debug!("收到答案块 #{}: {} 字符", chunk_count, text.text.len());
                            on_chunk(StreamChunk::Answer(text.text.clone()))?;
                            full_response.push_str(&text.text);
                        }
                        StreamedAssistantContent::ReasoningDelta { reasoning, .. } => {
                            chunk_count += 1;
                            debug!("收到推理块 #{}: {} 字符", chunk_count, reasoning.len());
                            on_chunk(StreamChunk::Reasoning(reasoning.clone()))?;
                        }
                        _ => {}
                    },
                    Ok(MultiTurnStreamItem::FinalResponse(_)) => {
                        debug!("收到最终响应标记");
                        break;
                    }
                    Ok(_) => {}
                    Err(e) => {
                        error!("流式请求出错: {}", e);
                        return Err(TaiError::AiError(format!("流式请求失败: {}", e)));
                    }
                }
            }
        }
    }

    debug!("流式请求完成，总共 {} 个块，响应长度: {} 字符", chunk_count, full_response.len());
    Ok(full_response)
}
