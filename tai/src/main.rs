use tai_command::Cli;
use clap::Parser;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli: Cli = Cli::parse();
    cli.handle().await
}
