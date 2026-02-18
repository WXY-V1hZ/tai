use clap::{Args, ValueHint};
use tai_ai::{chat_stream, load_active_model, load_providers, resolve_active, StreamChunk};
use tai_core::{TaiError, TaiResult};
use tai_tui::{ReasoningDisplay, Spinner};
use tracing::debug;

#[derive(Args, Debug)]
pub struct AskArgs {
    /// upload file
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub file: Option<String>,

    /// user requirement (if empty, enter editor)
    pub user_input: Option<String>,
}

impl AskArgs {
    pub async fn handle(self) -> TaiResult<()> {
        let prompt = self.user_input.ok_or(TaiError::EmptyInput)?;

        let final_prompt = match self.file {
            Some(path) => {
                debug!("附加文件: {}", path);
                format!("{}\n\n文件: {}", prompt, path)
            }
            None => prompt,
        };

        let providers = load_providers()?;
        if providers.is_empty() {
            return Err(TaiError::NoProviderConfig);
        }

        let active = load_active_model()?;
        let (provider, model) = resolve_active(&providers, active.as_ref())
            .ok_or(TaiError::NoActiveModel)?;

        debug!("使用模型: {}/{}", provider.provider, model);
        
        let spinner = Spinner::new("AI 思考中...");

        let mut display = ReasoningDisplay::new();
        let mut first_chunk = true;

        chat_stream(provider, model, &final_prompt, |chunk| {
            if first_chunk {
                spinner.finish_and_clear();
                first_chunk = false;
            }

            match chunk {
                StreamChunk::Reasoning(text) => {
                    debug!("推理块: {} 字符", text.len());
                    display.append_reasoning(&text);
                    display.render()?;
                }
                StreamChunk::Answer(text) => {
                    debug!("答案块: {} 字符", text.len());
                    display.append_answer(&text);
                    display.render()?;
                }
            }

            Ok(())
        })
        .await?;

        display.finish()?;
        debug!("Ask 命令完成");
        Ok(())
    }
}
