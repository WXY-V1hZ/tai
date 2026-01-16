use arboard::Clipboard;
use clap::Args;
use std::error::Error;

const PROMPT: &str = "\
你是一个命令行助手，你的任务如下；
1. 根据用户描述，生成对应的命令。
2. 直接返回对应的命令，不要有任何解释或额外文字。
3. 回答应为一行纯文本，不要用```包裹命令，保证用户复制你的回答能直接粘贴。
4. 使用一行流命令。
5. 如果用户想做的事情无法简单地通过一行命令解决，可以返回包括但不限于：文档地址、官网网址、博客网址等等，不要返回任何解释或额外文字。
6. 如果用户的描述与执行任务无关，则返回 ls 命令，第五条要求优先级高于第六条。
7. 第五条和第六条只能二选一进行返回
用户的描述为：
";

#[derive(Args, Debug)]
pub struct GoArgs {
    pub user_input: String,
}

impl GoArgs {
    pub async fn handle(self) -> Result<(), Box<dyn Error>> {
        // 构造提示词，让 AI 生成命令
        let prompt = format!("{} {}", PROMPT, self.user_input);

        // 获取 AI 建议
        let command = crate::ai::chat(&prompt).await?;

        // 打印建议的命令
        println!("{}", command);

        // 复制到剪贴板
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&command) {
                    eprintln!("警告: 无法复制到剪贴板: {}", e);
                } else {
                    println!("✓ 已复制到剪贴板");
                }
            }
            Err(e) => {
                eprintln!("警告: 无法访问剪贴板: {}", e);
            }
        }

        Ok(())
    }
}
