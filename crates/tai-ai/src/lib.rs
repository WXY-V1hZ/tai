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
use tokio::time::{sleep, Duration};
use tracing::{debug, error, warn};

const IS_TEST: bool = false;
const TEST_FILE: &str = "D:/program/proj/tai/test_response.md";
const TEST_DELAY_MS: u64 = 0;

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
    // 检查是否启用测试模式
    if IS_TEST {
        debug!("测试模式已启用");
        return chat_stream_test_mode(on_chunk).await;
    }

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

/// 测试模式：从本地文件读取并模拟流式输出
async fn chat_stream_test_mode<F>(mut on_chunk: F) -> TaiResult<String>
where
    F: FnMut(StreamChunk) -> TaiResult<()>,
{
    // 获取测试文件路径
    let test_file = TEST_FILE;
    
    debug!("测试模式：从文件读取响应: {}", test_file);
    
    // 读取文件内容
    let content = tokio::fs::read_to_string(&test_file).await.map_err(|e| {
        error!("无法读取测试文件 {}: {}", test_file, e);
        TaiError::FileError(format!("无法读取测试文件 {}: {}", test_file, e))
    })?;
    
    if content.is_empty() {
        warn!("测试文件为空");
        return Ok(String::new());
    }
    
    debug!("成功读取测试文件，内容长度: {} 字符", content.len());
    
    // 模拟流式输出，按行发送
    let mut full_response = String::new();
    let lines: Vec<&str> = content.lines().collect();
    
    for (i, line) in lines.iter().enumerate() {
        // 添加换行符（除了最后一行）
        let chunk_text = if i < lines.len() - 1 {
            format!("{}\n", line)
        } else {
            line.to_string()
        };
        
        // 发送块
        on_chunk(StreamChunk::Answer(chunk_text.clone()))?;
        full_response.push_str(&chunk_text);
        
        // 模拟网络延迟（可配置）
        sleep(Duration::from_millis(TEST_DELAY_MS)).await;
    }
    
    debug!("测试模式输出完成，总长度: {} 字符", full_response.len());
    Ok(full_response)
}
