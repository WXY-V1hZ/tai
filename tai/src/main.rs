use clap::Parser;
use tai_command::Cli;
use tai_core::init_logging;
use tracing::error;

#[tokio::main]
async fn main() {
    // 初始化日志系统
    init_logging();
    
    let cli: Cli = Cli::parse();
    
    if let Err(e) = cli.handle().await {
        error!("程序执行出错: {}", e);
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }
}
