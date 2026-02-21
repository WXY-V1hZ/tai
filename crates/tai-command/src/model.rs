use clap::{Args, Subcommand};
use tai_ai::{load_active_model, load_providers, resolve_active, save_active_model, save_providers, ActiveModel};
use tai_core::{TaiError, TaiResult};
use tai_tui::{config_providers, select_model, ModelItem, ProviderEntry};
use tracing::debug;

#[derive(Args, Debug)]
pub struct ModelArgs {
    /// 直接切换到指定模型（不触发 TUI）
    #[arg(short = 's', long = "switch")]
    pub switch: Option<String>,

    #[command(subcommand)]
    pub subcommand: Option<ModelSubcommand>,
}

#[derive(Subcommand, Debug)]
pub enum ModelSubcommand {
    /// 编辑各 Provider 的 API Key 和 Base URL
    Config,
}

impl ModelArgs {
    pub async fn handle(self) -> TaiResult<()> {
        if let Some(ModelSubcommand::Config) = self.subcommand {
            return handle_config();
        }

        let providers = load_providers()?;
        if providers.is_empty() {
            println!("未找到 provider 配置，请检查 ~/.tai/providers.json");
            return Ok(());
        }

        if let Some(model_name) = self.switch {
            return switch_model(&providers, &model_name);
        }

        // 无参数：触发交互式 TUI 列表
        let active = load_active_model()?;
        let current = resolve_active(&providers, active.as_ref());

        let items: Vec<ModelItem> = providers
            .iter()
            .flat_map(|p| {
                p.model_names
                    .iter()
                    .map(|m| ModelItem::new(&p.provider, m))
                    .collect::<Vec<_>>()
            })
            .collect();

        let current_index = current
            .and_then(|(cp, cm)| {
                items
                    .iter()
                    .position(|item| item.provider == cp.provider && item.model == cm)
            })
            .unwrap_or(0);

        debug!("打开交互式模型选择器，当前模型索引: {}", current_index);

        match select_model(&items, current_index) {
            Ok(Some(index)) => {
                let item = &items[index];
                let provider = providers
                    .iter()
                    .find(|p| p.provider == item.provider)
                    .unwrap();
                save_active_model(&ActiveModel {
                    provider: provider.provider.clone(),
                    model: item.model.clone(),
                })?;
                debug!("已切换到模型: {}/{}", provider.provider, item.model);
                println!("已切换到 {}/{}", provider.provider, item.model);
            }
            Ok(None) => {
                debug!("用户取消了模型选择");
            }
            Err(e) => {
                return Err(TaiError::AiError(format!("TUI 错误: {}", e)));
            }
        }

        Ok(())
    }
}

fn handle_config() -> TaiResult<()> {
    let providers = load_providers()?;
    if providers.is_empty() {
        println!("未找到 provider 配置，请检查 ~/.tai/providers.json");
        return Ok(());
    }

    let entries: Vec<ProviderEntry> = providers
        .iter()
        .map(|p| ProviderEntry {
            name: p.provider.clone(),
            base_url: p.base_url.clone(),
            api_key: p.api_key.clone(),
        })
        .collect();

    match config_providers(entries) {
        Ok(Some(updated)) => {
            let mut new_providers = providers.clone();
            for entry in &updated {
                if let Some(p) = new_providers.iter_mut().find(|p| p.provider == entry.name) {
                    p.base_url = entry.base_url.clone();
                    p.api_key = entry.api_key.clone();
                }
            }
            save_providers(&new_providers)?;
            debug!("Provider 配置已保存");
            println!("  ✓ 配置已保存至 ~/.tai/providers.json");
        }
        Ok(None) => {
            debug!("用户取消了 Provider 配置");
        }
        Err(e) => {
            return Err(TaiError::AiError(format!("TUI 错误: {}", e)));
        }
    }

    Ok(())
}

fn switch_model(providers: &[tai_ai::ProviderConfig], model_name: &str) -> TaiResult<()> {
    debug!("尝试切换到模型: {}", model_name);
    for provider in providers {
        if let Some(model) = provider.model_names.iter().find(|m| *m == model_name) {
            save_active_model(&ActiveModel {
                provider: provider.provider.clone(),
                model: model.clone(),
            })?;
            debug!("已切换到模型: {}/{}", provider.provider, model);
            println!("已切换到 {}/{}", provider.provider, model);
            return Ok(());
        }
    }
    Err(TaiError::ModelNotFound(model_name.to_string()))
}
