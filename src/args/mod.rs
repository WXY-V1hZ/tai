mod config;
mod do_ask;
mod go;
mod init;
mod model;

pub use config::ConfigCommand;
pub use do_ask::DoAskArgs;
pub use go::GoArgs;
pub use init::InitCommand;
pub use model::ModelArgs;

use clap::{Parser, Subcommand};
use std::error::Error;

#[derive(Parser, Debug)]
#[command(name = "tai", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// open directory as project (default: current directory)
    pub dir_path: Option<String>,
}

impl Cli {
    pub async fn handle(self) -> Result<(), Box<dyn Error>> {
        match self.command {
            Some(command) => command.handle().await,
            None => {
                let path = self.dir_path.unwrap_or_else(|| ".".to_string());
                println!("open directory: {}", path);
                Ok(())
            }
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Model(ModelArgs),
    Do(DoAskArgs),
    Ask(DoAskArgs),
    Go(GoArgs),
    Init,
    Config,
}

impl Commands {
    pub async fn handle(self) -> Result<(), Box<dyn Error>> {
        match self {
            Commands::Model(args) => args.handle().await,
            Commands::Do(args) => args.handle("do").await,
            Commands::Ask(args) => args.handle("ask").await,
            Commands::Go(args) => args.handle().await,
            Commands::Init => InitCommand.handle().await,
            Commands::Config => ConfigCommand.handle().await,
        }
    }
}
