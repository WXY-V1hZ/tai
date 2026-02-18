use dirs_next::home_dir;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderConfig {
    pub provider: String,
    pub base_url: String,
    pub api_key: String,
    pub model_names: Vec<String>,
}

/// 当前激活的提供商和模型，持久化到 ~/.tai/state.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveModel {
    pub provider: String,
    pub model: String,
}

fn tai_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".tai")
}

fn providers_path() -> PathBuf {
    tai_dir().join("providers.json")
}

fn state_path() -> PathBuf {
    tai_dir().join("state.json")
}

pub fn load_providers() -> Result<Vec<ProviderConfig>, Box<dyn Error>> {
    let path = providers_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn load_active_model() -> Result<Option<ActiveModel>, Box<dyn Error>> {
    let path = state_path();
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&content)?))
}

pub fn save_active_model(active: &ActiveModel) -> Result<(), Box<dyn Error>> {
    let path = state_path();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, serde_json::to_string_pretty(active)?)?;
    Ok(())
}

/// 根据状态文件找到对应的 (ProviderConfig, model_name)。
/// 如果状态不存在或已失效，回退到第一个可用的 provider + model。
pub fn resolve_active<'a>(
    providers: &'a [ProviderConfig],
    active: Option<&ActiveModel>,
) -> Option<(&'a ProviderConfig, &'a str)> {
    if let Some(a) = active {
        let found = providers
            .iter()
            .find(|p| p.provider == a.provider)
            .and_then(|p| {
                p.model_names
                    .iter()
                    .find(|m| m.as_str() == a.model)
                    .map(|m| (p, m.as_str()))
            });
        if found.is_some() {
            return found;
        }
    }
    // 回退：第一个 provider 的第一个 model
    providers
        .first()
        .and_then(|p| p.model_names.first().map(|m| (p, m.as_str())))
}
