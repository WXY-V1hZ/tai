mod ask;
mod config;
mod r#do;
mod go;
mod model;
mod provider;

pub use ask::AskArgs;
pub use config::ConfigCommand;
pub use r#do::DoArgs;
pub use go::GoArgs;
pub use model::ModelArgs;

use clap::{Parser, Subcommand};
use tai_core::TaiResult;
use tracing::debug;

#[derive(Parser, Debug)]
#[command(name = "tai", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// open directory as project (default: current directory)
    pub dir_path: Option<String>,
}

impl Cli {
    pub async fn handle(self) -> TaiResult<()> {
        match self.command {
            Some(command) => command.handle().await,
            None => {
                let path = self.dir_path.unwrap_or_else(|| ".".to_string());
                debug!("打开目录: {}", path);
                println!("open directory: {}", path);
                Ok(())
            }
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Model(ModelArgs),
    Do(DoArgs),
    Ask(AskArgs),
    Go(GoArgs),
    Config,
}

impl Commands {
    pub async fn handle(self) -> TaiResult<()> {
        match self {
            Commands::Model(args) => args.handle().await,
            Commands::Do(args) => args.handle().await,
            Commands::Ask(args) => args.handle().await,
            Commands::Go(args) => args.handle().await,
            Commands::Config => ConfigCommand.handle().await,
        }
    }
}
