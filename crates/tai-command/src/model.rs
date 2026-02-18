use clap::{Args, Subcommand};
use std::error::Error;
use tai_ai::{load_active_model, load_providers, resolve_active, save_active_model, ActiveModel};

#[derive(Args, Debug)]
pub struct ModelArgs {
    #[command(subcommand)]
    pub command: ModelCmd,
}

impl ModelArgs {
    pub async fn handle(self) -> Result<(), Box<dyn Error>> {
        self.command.handle().await
    }
}

#[derive(Subcommand, Debug)]
pub enum ModelCmd {
    /// list all available models from providers config
    List,

    /// switch active model (e.g. deepseek-chat, gpt-4o-mini)
    #[command(short_flag = 's')]
    Switch {
        #[arg(value_name = "MODEL")]
        model_name: String,
    },
}

impl ModelCmd {
    pub async fn handle(self) -> Result<(), Box<dyn Error>> {
        match self {
            ModelCmd::List => {
                let providers = load_providers()?;
                if providers.is_empty() {
                    println!("未找到 provider 配置，请检查 ~/.tai/providers.json");
                    return Ok(());
                }
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
            ModelCmd::Switch { model_name } => {
                let providers = load_providers()?;
                for provider in &providers {
                    if let Some(model) = provider.model_names.iter().find(|m| *m == &model_name) {
                        save_active_model(&ActiveModel {
                            provider: provider.provider.clone(),
                            model: model.clone(),
                        })?;
                        println!("已切换到 {}/{}", provider.provider, model);
                        return Ok(());
                    }
                }
                Err(format!("未找到模型 `{model_name}`，运行 `tai model list` 查看可用模型").into())
            }
        }
    }
}
