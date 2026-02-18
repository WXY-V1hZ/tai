use clap::{Args, ValueHint};
use tai_core::TaiResult;
use tracing::debug;

#[derive(Args, Debug)]
pub struct DoArgs {
    /// upload file
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    pub file: Option<String>,

    /// user requirement (if empty, enter editor)
    pub user_input: Option<String>,
}

impl DoArgs {
    pub async fn handle(self) -> TaiResult<()> {
        debug!("Do 命令被调用（功能开发中）");
        println!("该功能开发中，请耐心等待更新");
        Ok(())
    }
}
