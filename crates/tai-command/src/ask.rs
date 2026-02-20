use clap::{Args, ValueHint};
use tai_ai::{chat_stream, load_active_model, load_providers, resolve_active, StreamChunk};
use tai_core::{TaiError, TaiResult};
use tai_tui::{TextRenderer, Spinner};
use tracing::debug;

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
        // 如果指定了 -c 参数，显示历史记录
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

        let providers = load_providers()?;
        if providers.is_empty() {
            return Err(TaiError::NoProviderConfig);
        }

        let active = load_active_model()?;
        let (provider, model) = resolve_active(&providers, active.as_ref())
            .ok_or(TaiError::NoActiveModel)?;

        debug!("使用模型: {}/{}", provider.provider, model);
        
        let spinner = Spinner::new("AI 思考中...");

        let mut renderer = TextRenderer::new();
        let mut first_chunk = true;

        chat_stream(provider, model, &final_prompt, |chunk| {
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

        let markdown = renderer.finish()?;
        
        // 保存历史记录
        if !markdown.is_empty() {
            if let Err(e) = history::save_history(&markdown) {
                debug!("保存历史记录失败: {}", e);
            }
        }
        
        debug!("Ask 命令完成");
        Ok(())
    }
}
