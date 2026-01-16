use clap::{Args, ValueHint};
use std::error::Error;

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
        println!("{} command", command_type);
        // TODO: 实现 do/ask 命令逻辑
        Ok(())
    }
}
