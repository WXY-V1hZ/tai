---
name: tai-architecture
description: Tai 项目架构理解和模块职责指南。在需要了解项目结构、模块间依赖、添加新功能或重构代码时使用。包括 workspace 组织、各 crate 职责、关键技术实现。
---

# Tai 项目架构指南

## 项目概览

**名称**: Tai (态)  
**类型**: AI 驱动的命令行助手  
**语言**: Rust  
**架构**: Workspace 多模块架构

## Workspace 结构

```
tai/
├── Cargo.toml              # workspace 根配置
├── tai/                    # 主二进制 crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs         # 程序入口
├── crates/
│   ├── tai-command/        # 命令行接口
│   ├── tai-ai/             # AI 核心逻辑
│   ├── tai-tui/            # TUI 组件库
│   ├── tai-core/           # 公共工具库
│   └── tai-pty/            # PTY 终端模拟 (待开发)
└── README.md
```

## 模块职责和依赖关系

```
tai (主程序)
 ├─> tai-command (CLI 解析和命令处理)
 │    ├─> tai-ai (AI 交互)
 │    ├─> tai-tui (TUI 组件)
 │    └─> tai-core (工具库)
 ├─> tai-ai
 │    └─> tai-core
 ├─> tai-tui
 │    └─> tai-core
 └─> tai-core (基础库，无外部依赖)
```

## 各 Crate 详解

### tai-command

**职责**: 命令行接口，使用 `clap` 解析参数

**主要文件**:
- `lib.rs` - 定义 CLI 参数结构
- `ask.rs` - `tai ask` 命令处理
- `ask/history.rs` - 历史记录管理子模块
- `go.rs` - `tai go` 命令生成
- `do.rs` - `tai do` 命令执行
- `model.rs` - `tai model` 模型管理
- `init.rs` - `tai init` 系统信息收集
- `config.rs` - 配置文件管理

**关键实现**:

```rust
// CLI 参数结构
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Ask(AskArgs),     // AI 对话
    Go(GoArgs),       // 命令生成
    Model(ModelArgs), // 模型管理
    Init,             // 系统信息
}
```

**扩展新命令**:

1. 在 `Commands` 枚举添加新变体
2. 创建对应的参数结构
3. 在 `main.rs` 添加命令处理分支
4. 实现命令处理函数

### tai-ai

**职责**: AI 交互核心，管理多厂商 AI 客户端

**主要文件**:
- `lib.rs` - AI 客户端单例、流式输出核心
- `config.rs` - Provider 配置管理
- `provider.rs` - Provider 枚举定义

**关键技术**:

#### 单例模式

使用 `OnceLock` 实现线程安全的客户端单例：

```rust
static OPENAI_CLIENT: OnceLock<OpenAIClient> = OnceLock::new();
static DEEPSEEK_CLIENT: OnceLock<DeepSeekClient> = OnceLock::new();

fn get_openai_client() -> &'static OpenAIClient {
    OPENAI_CLIENT.get_or_init(|| {
        // 初始化逻辑
    })
}
```

**注意**: `rig-core` 的 OpenAI 客户端不兼容 DeepSeek，必须使用 DeepSeek 专用客户端。

#### 流式输出

```rust
pub async fn chat_stream<F1, F2>(
    on_reasoning: F1,
    on_answer: F2,
) -> Result<String>
where
    F1: Fn(&str) + Send + Sync,  // reasoning 回调
    F2: Fn(&str) + Send + Sync,  // answer 回调
{
    let mut stream = response.stream().await?;
    
    while let Some(chunk) = stream.next().await {
        if is_reasoning {
            on_reasoning(&chunk);
        } else {
            on_answer(&chunk);
        }
    }
}
```

#### 测试模式

```rust
const IS_TEST: bool = cfg!(feature = "test");

async fn chat_stream_test_mode() -> Result<String> {
    let content = fs::read_to_string("test_response.md")?;
    // 模拟流式输出
}
```

**添加新 Provider**:

1. 在 `provider.rs` 添加新枚举变体
2. 在 `lib.rs` 添加对应的客户端单例
3. 在 `chat_stream()` 添加客户端选择逻辑
4. 更新 `providers.json` 配置格式

### tai-tui

**职责**: TUI 组件库

**主要文件**:
- `reasoning.rs` - Markdown 流式渲染引擎
- `viewer.rs` - MadView 公共展示模块
- `model_selector.rs` - 交互式模型选择器
- `spinner.rs` - 加载动画

**关键技术**:

#### Markdown 渲染策略

```rust
pub struct ReasoningRenderer {
    reasoning_buffer: String,
    answer_buffer: String,
}

impl ReasoningRenderer {
    // 流式阶段：增量输出
    pub fn print_reasoning(&mut self, text: &str) {
        // 灰色输出 reasoning
    }
    
    pub fn print_answer(&mut self, text: &str) {
        // 原始输出 answer
    }
    
    // 完成阶段：美化展示
    pub fn finish(self) -> Result<String> {
        show_markdown_view(&self.answer_buffer, skin)?;
        Ok(self.answer_buffer)  // 只返回 answer
    }
}
```

#### 公共展示模块

```rust
// viewer.rs - 避免代码重复
pub fn show_markdown_view(markdown: &str, skin: MadSkin) -> Result<()> {
    execute!(stdout(), EnterAlternateScreen)?;
    
    let area = terminal_size()?;
    let mut view = MadView::from(markdown, area, skin);
    
    // 事件循环：上下键滚动，q/ESC 退出
    loop {
        view.write_on(&mut stdout())?;
        match read()? {
            Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                view.try_scroll_lines(-1);
            }
            // ...
        }
    }
    
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
```

**组件使用场景**:
- `reasoning.rs` - `tai ask` 命令的回答展示
- `viewer.rs` - 历史记录查看、任何 markdown 展示
- `model_selector.rs` - `tai model` 命令的模型选择
- `spinner.rs` - API 调用等待动画

### tai-core

**职责**: 公共工具库，无外部业务依赖

**主要文件**:
- `logging.rs` - 日志系统
- `error.rs` - 自定义错误类型
- `lib.rs` - 公共工具函数

**日志配置**:

```rust
pub fn init_logging() -> Result<()> {
    let log_dir = home_dir()?.join(".tai");
    
    let file_appender = tracing_appender::rolling::hourly(
        &log_dir,
        "tai.log"
    );
    
    // 终端: info, 文件: debug
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
```

**滚动策略**:
- 按小时滚动
- 最多保留 10 个文件
- 文件名: `tai-{timestamp}.log`

### tai-pty (待开发)

**职责**: PTY 终端模拟

**计划功能**:
- 命令执行虚拟终端
- ANSI 转义序列支持
- 输入输出重定向

## 配置文件

### providers.json

**位置**: `~/.tai/providers.json`

```json
[
  {
    "provider": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-...",
    "model_names": ["gpt-4o-mini", "gpt-4o"]
  },
  {
    "provider": "deepseek",
    "base_url": "https://api.deepseek.com",
    "api_key": "sk-...",
    "model_names": ["deepseek-chat", "deepseek-reasoner"]
  }
]
```

### active_model.txt

**位置**: `~/.tai/active_model.txt`

格式: `provider/model_name`

示例: `deepseek/deepseek-reasoner`

### 历史记录缓存

**位置**: `~/.tai/cache/history/`

文件格式: `YYYYMMDD_HHMMSS.md`

## 核心流程

### tai ask 命令流程

```
用户输入
  ↓
ask.rs: 解析参数 (-c, -f)
  ↓
[分支1: -c 参数] → history.rs: show_history()
  ↓
[分支2: 正常提问]
  ↓
ai.rs: chat_stream() - 调用 AI API
  ↓
reasoning.rs: 流式渲染
  ├─ reasoning: 灰色增量输出
  └─ answer: 原始 markdown 输出
  ↓
reasoning.rs: finish() - alternate screen 美化展示
  ↓
history.rs: save_history() - 自动保存
  ↓
返回结果
```

### tai model 命令流程

```
用户输入
  ↓
model.rs: 解析参数
  ↓
[分支1: 指定模型名] → 直接切换
  ↓
[分支2: 交互式选择]
  ↓
model_selector.rs: 显示模型列表
  ↓
用户上下键选择 + Enter 确认
  ↓
更新 active_model.txt
  ↓
显示切换成功信息
```

## 扩展指南

### 添加新功能

1. **确定功能位置**:
   - CLI 相关 → `tai-command`
   - AI 交互 → `tai-ai`
   - UI 组件 → `tai-tui`
   - 工具函数 → `tai-core`

2. **创建子模块**（如果功能复杂）:
   ```
   crates/tai-command/src/
   ├── new_feature.rs        # 主文件
   └── new_feature/          # 子模块
       ├── mod.rs
       ├── handler.rs
       └── utils.rs
   ```

3. **更新 CLI 定义**:
   在 `tai-command/src/lib.rs` 添加新命令

4. **实现功能逻辑**

5. **添加测试**

### 添加新 AI Provider

1. 在 `rig-core` 中查找对应的 Provider 实现
2. 如果没有，需要自定义客户端（参考 DeepSeek 实现）
3. 在 `tai-ai/src/provider.rs` 添加枚举变体
4. 在 `tai-ai/src/lib.rs` 添加客户端单例和初始化逻辑
5. 更新 `chat_stream()` 函数的 Provider 选择逻辑
6. 更新配置文件格式说明

### 优化 TUI 组件

1. **避免代码重复**: 提取公共逻辑到 `viewer.rs` 或新模块
2. **alternate screen 使用**: 对于全屏交互界面
3. **流式输出优化**: 平衡实时性和稳定性

## 依赖关系图

```
External Crates:
├── clap         → CLI 解析
├── rig-core     → AI 客户端
├── tokio        → 异步运行时
├── crossterm    → 终端控制
├── termimad     → Markdown 渲染
├── tracing      → 日志系统
├── chrono       → 时间处理
├── dirs         → 用户目录
└── anyhow       → 错误处理

Internal Crates:
tai (main)
 ├── tai-command
 │    ├── tai-ai
 │    │    └── tai-core
 │    ├── tai-tui
 │    │    └── tai-core
 │    └── tai-core
 └── tai-core
```

## 性能考虑

1. **单例客户端**: 避免重复初始化 AI 客户端
2. **流式输出**: 不等待完整响应，边接收边展示
3. **缓冲写入**: 文件操作使用 `BufWriter`
4. **异步处理**: 所有 IO 操作异步化

## 常见架构问题

### 1. 模块间循环依赖

**解决**: 提取公共逻辑到 `tai-core`，或调整依赖方向

### 2. 重复代码

**解决**: 提取到公共模块（如 `viewer.rs`）

### 3. 文件过大

**解决**: 创建子模块（如 `ask/history.rs`）

### 4. 测试困难

**解决**: 使用依赖注入，提供 mock 实现
