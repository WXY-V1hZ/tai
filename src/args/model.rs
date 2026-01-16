use clap::{Args, Subcommand};
use std::error::Error;

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
    /// list enabled models
    List,

    /// switch model
    #[command(short_flag = 'c')]
    Switch {
        #[arg(value_name = "MODEL")]
        model_name: String,
    },
}

impl ModelCmd {
    pub async fn handle(self) -> Result<(), Box<dyn Error>> {
        match self {
            ModelCmd::List => {
                println!("list enabled models");
                Ok(())
            }
            ModelCmd::Switch { model_name } => {
                println!("switch model to: {}", model_name);
                Ok(())
            }
        }
    }
}
