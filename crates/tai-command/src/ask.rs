use clap::{Args, ValueHint};
use tai_ai::{chat_stream, ProviderConfig, StreamChunk};
use tai_core::{TaiConfig, TaiError, TaiResult};
use tai_tui::{Spinner, TextRenderer};
use tracing::debug;

use crate::provider::{ensure_active_provider, recover_auth_error};

mod history;
use history::show_history;

#[derive(Args, Debug)]
pub struct AskArgs {
    /// upload file
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub file: Option<String>,

    /// view cached history (number of records to show, default: 1 for last response)
    #[arg(short, long, num_args = 0..=1, default_missing_value = "1")]
    pub cache: Option<usize>,

    /// user requirement (if empty, enter editor)
    pub user_input: Option<String>,
}

impl AskArgs {
    pub async fn handle(self) -> TaiResult<()> {
        if let Some(count) = self.cache {
            return show_history(count);
        }

        let prompt = self.user_input.ok_or(TaiError::EmptyInput)?;

        let final_prompt = match self.file {
            Some(path) => {
                debug!("附加文件: {}", path);
                format!("{}\n\n文件: {}", prompt, path)
            }
            None => prompt,
        };

        let config = TaiConfig::load().unwrap_or_default();
        let mut context = ensure_active_provider().await?;

        loop {
            debug!("使用模型: {}/{}", context.0.provider, context.1);
            match do_ask(&context.0, &context.1, &final_prompt, &config).await {
                Ok(markdown) => {
                    if !markdown.is_empty() {
                        if let Err(e) = history::save_history(&markdown) {
                            debug!("保存历史记录失败: {}", e);
                        }
                    }
                    debug!("Ask 命令完成");
                    return Ok(());
                }
                Err(TaiError::AuthError(ref name)) => {
                    context = recover_auth_error(name).await?;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

async fn do_ask(
    provider: &ProviderConfig,
    model: &str,
    prompt: &str,
    config: &TaiConfig,
) -> TaiResult<String> {
    let spinner = Spinner::new("AI 思考中...");
    let mut renderer = TextRenderer::new();
    let mut first_chunk = true;

    chat_stream(provider, model, prompt, |chunk| {
        if first_chunk {
            spinner.finish_and_clear();
            first_chunk = false;
        }
        match chunk {
            StreamChunk::Reasoning(text) => {
                debug!("推理块: {} 字符", text.len());
                renderer.append_reasoning(&text);
                renderer.render()?;
            }
            StreamChunk::Answer(text) => {
                debug!("答案块: {} 字符", text.len());
                renderer.append_answer(&text);
                renderer.render()?;
            }
        }
        Ok(())
    })
    .await?;

    Ok(renderer.finish(config.show_markdown_view)?)
}
