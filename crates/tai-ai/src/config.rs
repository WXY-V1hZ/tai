use dirs_next::home_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tai_core::{TaiError, TaiResult};
use tracing::{debug, error, warn};

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

const DEFAULT_PROVIDERS: &str = include_str!("../../../assets/providers.json");

pub fn load_providers() -> TaiResult<Vec<ProviderConfig>> {
    let path = providers_path();
    debug!("加载 provider 配置: {:?}", path);

    if !path.exists() {
        warn!("Provider 配置文件不存在，创建默认配置: {:?}", path);
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir).map_err(|e| {
            error!("创建 .tai 目录失败: {}", e);
            TaiError::FileError(format!("无法创建目录 {:?}: {}", dir, e))
        })?;
        fs::write(&path, DEFAULT_PROVIDERS).map_err(|e| {
            error!("写入默认 provider 配置失败: {}", e);
            TaiError::FileError(format!("无法写入 {:?}: {}", path, e))
        })?;
        debug!("默认 provider 配置已写入: {:?}", path);
    }

    let content = fs::read_to_string(&path).map_err(|e| {
        error!("读取 provider 配置失败: {}", e);
        TaiError::FileError(format!("无法读取 {:?}: {}", path, e))
    })?;

    let providers: Vec<ProviderConfig> = serde_json::from_str(&content)?;
    debug!("成功加载 {} 个 provider 配置", providers.len());

    Ok(providers)
}

pub fn load_active_model() -> TaiResult<Option<ActiveModel>> {
    let path = state_path();
    debug!("加载激活模型状态: {:?}", path);
    
    if !path.exists() {
        debug!("状态文件不存在，使用默认模型");
        return Ok(None);
    }
    
    let content = fs::read_to_string(&path).map_err(|e| {
        error!("读取状态文件失败: {}", e);
        TaiError::FileError(format!("无法读取 {:?}: {}", path, e))
    })?;
    
    let active: ActiveModel = serde_json::from_str(&content)?;
    debug!("加载激活模型: {}/{}", active.provider, active.model);
    
    Ok(Some(active))
}

pub fn save_active_model(active: &ActiveModel) -> TaiResult<()> {
    let path = state_path();
    debug!("保存激活模型: {}/{} 到 {:?}", active.provider, active.model, path);
    
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| {
        error!("创建目录失败: {}", e);
        TaiError::FileError(format!("无法创建目录: {}", e))
    })?;
    
    let content = serde_json::to_string_pretty(active)?;
    fs::write(&path, content).map_err(|e| {
        error!("写入状态文件失败: {}", e);
        TaiError::FileError(format!("无法写入 {:?}: {}", path, e))
    })?;
    
    debug!("成功保存激活模型: {}/{}", active.provider, active.model);
    Ok(())
}

pub fn save_providers(providers: &[ProviderConfig]) -> TaiResult<()> {
    let path = providers_path();
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| {
        error!("创建目录失败: {}", e);
        TaiError::FileError(format!("无法创建目录: {}", e))
    })?;
    let content = serde_json::to_string_pretty(providers)?;
    fs::write(&path, content).map_err(|e| {
        error!("写入 provider 配置失败: {}", e);
        TaiError::FileError(format!("无法写入 {:?}: {}", path, e))
    })?;
    debug!("已保存 {} 个 provider 配置", providers.len());
    Ok(())
}

pub fn update_provider_api_key(provider_name: &str, api_key: &str) -> TaiResult<()> {
    let mut providers = load_providers()?;
    let p = providers
        .iter_mut()
        .find(|p| p.provider == provider_name)
        .ok_or_else(|| {
            TaiError::ConfigError(format!("Provider '{}' 不存在于配置文件中", provider_name))
        })?;
    p.api_key = api_key.to_string();
    debug!("已更新 {} 的 API Key", provider_name);
    save_providers(&providers)
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
            debug!("使用配置的激活模型: {}/{}", a.provider, a.model);
            return found;
        } else {
            warn!("配置的激活模型 {}/{} 不存在，使用回退", a.provider, a.model);
        }
    }
    
    // 回退：第一个 provider 的第一个 model
    let fallback = providers
        .first()
        .and_then(|p| p.model_names.first().map(|m| (p, m.as_str())));
    
    if let Some((p, m)) = fallback {
        debug!("使用回退模型: {}/{}", p.provider, m);
    }
    
    fallback
}
