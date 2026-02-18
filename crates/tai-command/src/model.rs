use clap::Args;
use tai_ai::{load_active_model, load_providers, resolve_active, save_active_model, ActiveModel};
use tai_core::{TaiError, TaiResult};
use tracing::debug;

#[derive(Args, Debug)]
pub struct ModelArgs {
    /// model name to switch (e.g. deepseek-chat, gpt-4o-mini)
    pub model_name: Option<String>,
}

impl ModelArgs {
    pub async fn handle(self) -> TaiResult<()> {
        let providers = load_providers()?;
        if providers.is_empty() {
            println!("未找到 provider 配置，请检查 ~/.tai/providers.json");
            return Ok(());
        }

        match self.model_name {
            Some(model_name) => {
                debug!("尝试切换到模型: {}", model_name);
                
                // 切换模型
                for provider in &providers {
                    if let Some(model) = provider.model_names.iter().find(|m| *m == &model_name) {
                        save_active_model(&ActiveModel {
                            provider: provider.provider.clone(),
                            model: model.clone(),
                        })?;
                        println!("已切换到 {}/{}", provider.provider, model);
                        debug!("模型切换成功: {}/{}", provider.provider, model);
                        return Ok(());
                    }
                }
                
                Err(TaiError::ModelNotFound(model_name))
            }
            None => {
                debug!("列出所有可用模型");
                
                // 列出所有模型
                let active = load_active_model()?;
                let current = resolve_active(&providers, active.as_ref());

                for provider in &providers {
                    for model in &provider.model_names {
                        let is_active = current
                            .map(|(p, m)| p.provider == provider.provider && m == model.as_str())
                            .unwrap_or(false);
                        let marker = if is_active { "*" } else { " " };
                        println!("{} {}/{}", marker, provider.provider, model);
                    }
                }
                Ok(())
            }
        }
    }
}
