use clap::{Args, ValueHint};
use std::error::Error;
use tai_ai::{chat_stream, load_active_model, load_providers, resolve_active, StreamChunk};
use tai_tui::{ReasoningDisplay, Spinner};

#[derive(Args, Debug)]
pub struct DoAskArgs {
    /// upload file
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub file: Option<String>,

    /// user requirement (if empty, enter editor)
    pub user_input: Option<String>,
}

impl DoAskArgs {
    pub async fn handle(self, command_type: &str) -> Result<(), Box<dyn Error>> {
        if command_type == "ask" {
            let prompt = self.user_input.ok_or("用户输入不能为空")?;

            let final_prompt = match self.file {
                Some(path) => format!("{}\n\n文件: {}", prompt, path),
                None => prompt,
            };
    
            let providers = load_providers()?;
            if providers.is_empty() {
                return Err("未找到 provider 配置，请检查 ~/.tai/providers.json".into());
            }
    
            let active = load_active_model()?;
            let (provider, model) = resolve_active(&providers, active.as_ref())
                .ok_or("没有可用的模型，请运行 `tai model list` 确认配置")?;
    
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
                        display.append_reasoning(&text);
                        display.render()?;
                    }
                    StreamChunk::Answer(text) => {
                        display.append_answer(&text);
                        display.render()?;
                    }
                }
    
                Ok(())
            })
            .await?;
    
            display.finish()?;
            Ok(())
        } else {
            println!("该功能开发中，请耐心等待更新");
            Ok(())
        }
    }
}
