use std::fmt::Debug;
use std::error::Error;

#[derive(Debug)]
pub struct ConfigCommand;

impl ConfigCommand {
    pub async fn handle(self) -> Result<(), Box<dyn Error>> {
        println!("config");
        // TODO: 实现 config 命令逻辑
        Ok(())
    }
}
