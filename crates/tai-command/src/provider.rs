use tai_ai::{load_active_model, load_providers, resolve_active, update_provider_api_key, ProviderConfig};
use tai_core::{TaiError, TaiResult};
use tai_tui::prompt_api_key;
use tracing::debug;

/// API Key 认证失败后，清空旧 key 并重新引导用户输入
pub async fn recover_auth_error(provider_name: &str) -> TaiResult<(ProviderConfig, String)> {
    eprintln!("\n  ✗ API Key 认证失败，请重新输入");
    update_provider_api_key(provider_name, "")?;
    ensure_active_provider().await
}

/// 解析当前激活的 provider 和模型，若 API Key 为空则引导用户填写
/// 返回所有权的 (ProviderConfig, model_name)
pub async fn ensure_active_provider() -> TaiResult<(ProviderConfig, String)> {
    let providers = load_providers()?;
    if providers.is_empty() {
        return Err(TaiError::NoProviderConfig);
    }

    let active = load_active_model()?;
    let (provider, model) = resolve_active(&providers, active.as_ref())
        .ok_or(TaiError::NoActiveModel)?;

    if !provider.api_key.is_empty() {
        return Ok((provider.clone(), model.to_string()));
    }

    debug!("Provider {} 的 API Key 为空，触发引导输入", provider.provider);

    let provider_name = provider.provider.clone();
    let api_key = prompt_api_key(&provider_name).map_err(|e| {
        TaiError::Other(format!("TUI 错误: {}", e))
    })?;

    match api_key {
        None => Err(TaiError::Other("已取消，请配置 API Key 后重试".to_string())),
        Some(key) => {
            update_provider_api_key(&provider_name, &key)?;
            debug!("API Key 已保存: {}", provider_name);
            println!("  ✓ API Key 已保存至 ~/.tai/providers.json");

            // 重新加载以获取更新后的配置
            let updated_providers = load_providers()?;
            let (updated_provider, updated_model) =
                resolve_active(&updated_providers, active.as_ref())
                    .ok_or(TaiError::NoActiveModel)?;

            Ok((updated_provider.clone(), updated_model.to_string()))
        }
    }
}
