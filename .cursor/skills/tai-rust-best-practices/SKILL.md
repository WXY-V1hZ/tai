---
name: tai-rust-best-practices
description: Tai 项目 Rust 代码编写最佳实践和约束。在编写或修改 Tai 项目代码时使用，包括代码结构、模块划分、错误处理、日志系统等规范。
---

# Tai 项目 Rust 代码编写最佳实践

## 核心原则

1. **文件行数限制**: 每个代码文件不应超过 500 行
   - 超过时应提取函数或对象到单独文件
   - 使用子模块（如 `ask/history.rs`）组织代码

2. **代码可读性优先**: 变量命名清晰，逻辑结构简洁
3. **代码复用**: 提取公共逻辑到独立模块（如 `viewer.rs`）
4. **避免不必要的注释**: 只解释非显而易见的意图，不要叙述代码功能

## 模块组织规范

### 子模块创建

当功能复杂度增加时，创建子模块：

```rust
// crates/tai-command/src/ask.rs
mod history;  // 对应 ask/history.rs
pub use history::{save_history, show_history};
```

### 公共模块提取

发现重复代码时立即提取：

```rust
// crates/tai-tui/src/viewer.rs - 公共展示逻辑
pub fn show_markdown_view(markdown: &str, skin: MadSkin) -> Result<()> {
    // alternate screen 展示逻辑
}
```

## 错误处理

### 使用 `thiserror` 定义错误类型

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("API error: {0}")]
    Api(String),
}
```

### 错误传播策略

- **核心功能**: 使用 `?` 传播错误，让调用者处理
- **辅助功能**: 使用日志记录错误，不中断主流程

```rust
// 历史记录保存失败不影响主流程
if let Err(e) = save_history(&answer) {
    tracing::warn!("Failed to save history: {}", e);
}
```

## 日志系统

### 使用 `tracing` 框架

```rust
use tracing::{info, warn, error, debug};

// info: 重要操作记录
tracing::info!("Saving history to {:?}", file_path);

// warn: 非致命错误
tracing::warn!("Failed to cleanup old history: {}", e);

// error: 严重错误
tracing::error!("API call failed: {}", e);

// debug: 调试信息（仅文件日志）
tracing::debug!("Received chunk: {:?}", chunk);
```

### 日志级别配置

- **终端输出**: `info` 及以上
- **文件记录**: `debug` 及以上
- **位置**: `~/.tai/tai-{timestamp}.log`

## 异步编程规范

### 使用 Tokio 运行时

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 异步主函数
}
```

### 流式处理模式

```rust
let mut stream = response.stream().await?;
while let Some(chunk) = stream.next().await {
    // 处理流式数据
    callback(&chunk)?;
}
```

## 单例模式

使用 `OnceLock` 实现线程安全的单例：

```rust
use std::sync::OnceLock;

static CLIENT: OnceLock<OpenAIClient> = OnceLock::new();

pub fn get_client() -> &'static OpenAIClient {
    CLIENT.get_or_init(|| {
        // 初始化逻辑
    })
}
```

## 配置管理

### 配置文件位置

所有配置文件存放在 `~/.tai/`:
- `providers.json` - AI 提供商配置
- `active_model.txt` - 当前活动模型
- `cache/history/` - 历史记录缓存

### 配置读取示例

```rust
use dirs::home_dir;
use std::fs;

let config_dir = home_dir()
    .ok_or_else(|| anyhow!("Cannot find home directory"))?
    .join(".tai");

let providers = fs::read_to_string(config_dir.join("providers.json"))?;
```

## 命令行参数

### 使用 `clap` 4.x derive API

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "tai")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    Ask(AskArgs),
    Model(ModelArgs),
}
```

### 可选参数处理

```rust
// 支持 -c 不带值默认为 1
#[arg(short, long, num_args = 0..=1, default_missing_value = "1")]
pub cache: Option<usize>,
```

## 文件操作

### 使用 `chrono` 生成时间戳

```rust
use chrono::Local;

let timestamp = Local::now().format("%Y%m%d_%H%M%S");
let filename = format!("{}.md", timestamp);
```

### 文件清理策略

```rust
// 按修改时间排序，保留最新 N 条
entries.sort_by(|a, b| b.modified.cmp(&a.modified));
for entry in entries.iter().skip(MAX_COUNT) {
    fs::remove_file(&entry.path)?;
}
```

## TUI 开发

### 使用 `crossterm` 控制终端

```rust
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

// 进入 alternate screen
execute!(std::io::stdout(), EnterAlternateScreen)?;

// 退出时恢复
execute!(std::io::stdout(), LeaveAlternateScreen)?;
```

### Markdown 渲染策略

- **流式阶段**: 原始文本增量输出（避免闪烁）
- **完成阶段**: `termimad` 美化渲染（alternate screen）

```rust
// 流式阶段：简单打印
print!("{}", chunk);

// 完成阶段：美化展示
show_markdown_view(&markdown, skin)?;
```

## 性能优化

1. **避免不必要的克隆**: 使用引用传递
2. **缓冲输出**: 使用 `BufWriter` 写文件
3. **流式处理**: 不要一次性加载所有数据到内存

```rust
use std::io::BufWriter;

let file = File::create(path)?;
let mut writer = BufWriter::new(file);
writer.write_all(content.as_bytes())?;
```

## 依赖管理

### 主要依赖版本

- `rig-core`: 0.28.0 - AI 客户端
- `clap`: 4.5.59 - 命令行解析
- `tokio`: 1.49.0 - 异步运行时
- `crossterm`: 0.28/0.29 - 终端控制
- `termimad`: 0.31 - Markdown 渲染
- `tracing`: 0.1 - 日志系统
- `chrono`: 0.4 - 时间处理
- `dirs`: 5.0 - 用户目录

### Workspace 依赖管理

在根 `Cargo.toml` 定义公共依赖版本：

```toml
[workspace.dependencies]
anyhow = "1.0"
tokio = { version = "1.49.0", features = ["full"] }
```

子 crate 引用：

```toml
[dependencies]
anyhow.workspace = true
tokio.workspace = true
```

## 测试

### 测试模式实现

```rust
// 使用环境变量控制测试模式
const IS_TEST: bool = cfg!(feature = "test");

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_chat_stream() {
        // 测试逻辑
    }
}
```

### 测试文件

项目根目录的 `test_*.md` 文件用于手动测试，不提交到版本控制。

## 代码审查清单

提交代码前检查：

- [ ] 文件不超过 500 行
- [ ] 重复代码已提取为公共模块
- [ ] 错误处理得当（核心功能传播，辅助功能记录日志）
- [ ] 添加了适当的日志记录
- [ ] 配置文件路径使用 `~/.tai/`
- [ ] 异步函数使用 `async/await`
- [ ] 没有不必要的代码注释
- [ ] `cargo build` 无警告
- [ ] `cargo clippy` 无警告

## 常见问题

### 1. ANSI 转义序列清理

如果输出包含 ANSI 标记，使用正则或 `strip-ansi-escapes` crate：

```rust
fn strip_ansi(s: &str) -> String {
    // 简单正则：\x1b\[[0-9;]*m
    s.replace("\x1b[0m", "").replace("\x1b[m", "")
}
```

### 2. 跨平台路径

使用 `std::path::Path` 和 `PathBuf`，避免硬编码路径分隔符：

```rust
let path = base_dir.join("cache").join("history");
```

### 3. 字符串处理

优先使用 `&str`，只在需要所有权时使用 `String`：

```rust
fn process(text: &str) -> String {  // 参数用 &str
    text.to_uppercase()             // 返回 String
}
```
