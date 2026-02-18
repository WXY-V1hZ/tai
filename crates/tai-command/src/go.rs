use arboard::Clipboard;
use clap::Args;
use tai_ai::{chat, load_active_model, load_providers, resolve_active};
use tai_core::{TaiError, TaiResult};
use tai_tui::Spinner;
use tracing::{debug, warn};

const PROMPT: &str = "\
你是一名命令行助手，请严格遵循以下规则：

【核心任务】
根据用户描述生成对应的命令行命令。

【输出规范】
1. 只返回命令本身，无任何解释、注释或额外文字
2. 单行纯文本，不使用 ``` 或其他标记包裹
3. 优先使用一行流命令（管道、链式操作）

【优先级规则】
1. 若任务无法通过单行命令简单完成 → 返回相关文档/官网/博客链接（仅URL）
2. 若用户描述与命令执行无关 → 返回：ls
3. 以上两条互斥，仅满足其一时执行

【安全约束】
- 不生成危险命令（如 rm -rf /、格式化磁盘等）
- 不生成需要交互式确认的命令
- 涉及系统修改时优先使用安全替代方案

【示例】
用户：列出当前目录下所有 .txt 文件
返回：find . -name \"*.txt\"

用户：如何配置 Kubernetes 集群
返回：https://kubernetes.io/docs/setup/

用户：今天天气怎么样
返回：ls

【用户描述】
";

#[derive(Args, Debug)]
pub struct GoArgs {
    pub user_input: String,
}

impl GoArgs {
    pub async fn handle(self) -> TaiResult<()> {
        debug!("Go 命令: 用户输入 = {}", self.user_input);
        
        let providers = load_providers()?;
        if providers.is_empty() {
            return Err(TaiError::NoProviderConfig);
        }
        
        let active = load_active_model()?;
        let (provider, model) = resolve_active(&providers, active.as_ref())
            .ok_or(TaiError::NoActiveModel)?;

        let prompt = format!("{} {}", PROMPT, self.user_input);

        let spinner = Spinner::new("AI 思考中...");
        let command = chat(provider, model, &prompt).await?;
        spinner.finish_and_clear();
        
        println!("{}", command);

        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&command) {
                    warn!("无法复制到剪贴板: {}", e);
                    eprintln!("警告: 无法复制到剪贴板: {}", e);
                } else {
                    debug!("命令已复制到剪贴板");
                    println!("✓ 已复制到剪贴板");
                }
            }
            Err(e) => {
                warn!("无法访问剪贴板: {}", e);
                eprintln!("警告: 无法访问剪贴板: {}", e);
            }
        }

        debug!("Go 命令完成");
        Ok(())
    }
}
