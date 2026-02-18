use rig::providers::{deepseek, openai};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use crate::config::ProviderConfig;

#[derive(Clone)]
pub enum AiClient {
    OpenAI(openai::Client),
    DeepSeek(deepseek::Client),
}

static CLIENT_REGISTRY: OnceLock<RwLock<HashMap<String, AiClient>>> = OnceLock::new();

fn registry() -> &'static RwLock<HashMap<String, AiClient>> {
    CLIENT_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// 按 (provider, base_url, api_key) 复用 client，不存在时惰性创建
pub fn get_client(config: &ProviderConfig) -> AiClient {
    let key = format!("{}|{}|{}", config.provider, config.base_url, config.api_key);

    {
        let read = registry().read().expect("CLIENT_REGISTRY read lock poisoned");
        if let Some(client) = read.get(&key) {
            return client.clone();
        }
    }

    let mut write = registry()
        .write()
        .expect("CLIENT_REGISTRY write lock poisoned");
    write
        .entry(key)
        .or_insert_with(|| match config.provider.to_lowercase().as_str() {
            "openai" => AiClient::OpenAI(
                openai::Client::builder()
                    .base_url(&config.base_url)
                    .api_key(&config.api_key)
                    .build()
                    .expect("Failed to build OpenAI client"),
            ),
            "deepseek" => AiClient::DeepSeek(
                deepseek::Client::builder()
                    .base_url(&config.base_url)
                    .api_key(&config.api_key)
                    .build()
                    .expect("Failed to build DeepSeek client"),
            ),
            _ => panic!("Unsupported provider: {}", config.provider),
        })
        .clone()
}
