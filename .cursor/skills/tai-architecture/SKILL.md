---
name: tai-architecture
description: Tai 项目架构理解和模块职责指南。在需要了解项目结构、模块间依赖、添加新功能或重构代码时使用。包括 workspace 组织、各 crate 职责、关键技术实现。
---

# Tai 项目架构指南

## Workspace 结构

```
tai/
├── Cargo.toml
├── assets/
│   └── providers.json          # 默认 provider 配置模板（编译期嵌入）
├── tai/src/main.rs             # 主二进制入口
└── crates/
    ├── tai-command/            # CLI 命令处理
    ├── tai-ai/                 # AI 核心逻辑
    ├── tai-tui/                # TUI 组件库
    └── tai-core/               # 公共工具库
```

## 依赖关系

```
tai-command ──→ tai-ai ──→ tai-core
            ──→ tai-tui
            ──→ tai-core
tai-tui 无内部依赖（tai-core 除外不依赖其他内部 crate）
```

## 各 Crate 详解

### tai-command

**职责**: CLI 参数解析 + 命令编排

**主要文件**:
- `lib.rs` - Commands 枚举定义
- `provider.rs` - `ensure_active_provider()` / `recover_auth_error()`
- `ask.rs` - `tai ask`（含 auth 重试循环）
- `go.rs` - `tai go`（含 auth 重试循环）
- `model.rs` - `tai model` / `tai model config`
- `config.rs` - `tai config`（TaiConfig ↔ SettingItem 转换）
- `ask/history.rs` - 历史记录子模块

**当前 CLI 结构**:

```rust
enum Commands {
    Ask(AskArgs),          // AI 对话
    Go(GoArgs),            // 命令生成
    Model(ModelArgs),      // 模型管理（含 Config 子命令）
    Config,                // 应用配置
    Do(DoArgs),            // 执行命令（开发中）
}

// tai model config 子命令
enum ModelSubcommand {
    Config,
}
```

**provider.rs 关键模式**:

```rust
// 解析激活 provider，api_key 为空时触发 TUI 引导
pub async fn ensure_active_provider() -> TaiResult<(ProviderConfig, String)>

// 认证失败后清空 key 并重新引导
pub async fn recover_auth_error(provider_name: &str) -> TaiResult<(ProviderConfig, String)>
```

**ask.rs / go.rs Auth 重试循环**:

```rust
let mut context = ensure_active_provider().await?;
loop {
    match do_ask(&context.0, &context.1, &prompt, &config).await {
        Ok(markdown) => { /* 保存历史 */ return Ok(()); }
        Err(TaiError::AuthError(ref name)) => {
            context = recover_auth_error(name).await?;
        }
        Err(e) => return Err(e),
    }
}
```

---

### tai-ai

**职责**: AI 交互核心，多厂商客户端管理

**主要文件**:
- `lib.rs` - `chat()` / `chat_stream()` + 错误分类
- `config.rs` - Provider 配置的 load/save
- `provider.rs` - `AiClient` 枚举，OnceLock 客户端单例

**错误分类** (`classify_error` in `lib.rs`):

```rust
fn classify_error(err: &str, provider: &ProviderConfig) -> TaiError {
    if 包含 "401" / "authentication" / "unauthorized" → TaiError::AuthError(provider_name)
    if 包含 "connect" / "dns" / "timed out"          → TaiError::ConnectionError(base_url)
    else                                               → TaiError::AiError(msg)
}
```

**config.rs 关键函数**:
- `load_providers()` — 不存在时用 `include_str!("../../../assets/providers.json")` 自动创建
- `save_providers(&[ProviderConfig])` — 通用保存（`update_provider_api_key` 基于此实现）
- `update_provider_api_key(name, key)` — 更新单个 provider 的 api_key

---

### tai-tui

**职责**: TUI 组件库，**不依赖 tai-ai**（使用自定义数据结构避免循环依赖）

**文件及组件**:

| 文件 | 导出 | 用途 |
|------|------|------|
| `reasoning.rs` | `TextRenderer` | 流式渲染 + `finish(render_markdown: bool)` |
| `viewer.rs` | `show_markdown_view` | alternate screen Markdown 渲染 |
| `model_selector.rs` | `select_model`, `ModelItem` | 模型选择列表 |
| `api_key_input.rs` | `prompt_api_key` | API Key 输入（掩码，增量渲染） |
| `provider_config.rs` | `config_providers`, `ProviderEntry` | Provider 三屏编辑 TUI |
| `settings.rs` | `show_settings`, `SettingItem`, `SettingValue` | 应用配置设置 TUI |
| `spinner.rs` | `Spinner` | 加载动画 |

**TUI 渲染模式**:

```
非全屏（model_selector, api_key_input, provider_config）:
  - 记录 start_row，用 Clear(FromCursorDown) 重绘
  - 文字输入类：draw_static() 只绘一次，按键时只更新输入行（避免闪烁）

全屏（settings, show_markdown_view）:
  - EnterAlternateScreen + Clear(All) + MoveTo(0,0)
  - 彻底避免底部追加问题
  - raw_mode 必须在 EnterAlternateScreen 之前启用（Windows 要求）
  - 进入后调用 flush_pending_events() 清除积压事件
```

**SettingValue 类型**（`settings.rs`）:

```rust
pub enum SettingValue {
    Bool(bool),                                    // Space 切换，●/○ 显示
    Select { options: Vec<String>, selected },      // Space 循环，◈ 显示
    Int { value: i64, min: i64, max: i64 },        // ← → 调整，‹ n › 显示
}
```

**TextRenderer**（`reasoning.rs`）:

```rust
// 流式阶段
renderer.append_reasoning(&text); renderer.render()?;
renderer.append_answer(&text);    renderer.render()?;

// 完成阶段：render_markdown 由 TaiConfig.show_markdown_view 控制
let markdown = renderer.finish(config.show_markdown_view)?;
```

---

### tai-core

**职责**: 公共工具库

**主要文件**:
- `error.rs` - `TaiError` 枚举（含 `AuthError`, `ConnectionError`）
- `logging.rs` - 双层日志（控制台无时间戳 `.without_time()`，文件含时间戳）
- `config.rs` - `TaiConfig` 结构体（load/save `~/.tai/config.json`）

**TaiError 关键变体**:

```rust
AuthError(String)       // provider_name，触发 API Key 重新输入
ConnectionError(String) // base_url，提示检查 base_url 配置
```

**TaiConfig**（`~/.tai/config.json`，不存在时使用默认值）:

```rust
pub struct TaiConfig {
    pub show_markdown_view: bool,  // ask 后是否展示 Markdown 渲染界面
    pub auto_copy_command: bool,
    pub save_history: bool,
    pub show_reasoning: bool,
    pub compact_output: bool,
    pub debug_logging: bool,
    pub max_history_count: u32,
    pub output_theme: String,
}
```

---

## 配置文件

| 文件 | 位置 | 说明 |
|------|------|------|
| `providers.json` | `~/.tai/providers.json` | Provider 列表，api_key 为空时触发 TUI |
| `state.json` | `~/.tai/state.json` | 当前激活的 provider/model |
| `config.json` | `~/.tai/config.json` | 应用配置，缺失时用默认值 |
| 历史记录 | `~/.tai/cache/history/` | `YYYYMMDD_HHMMSS.md` |

**providers.json** `api_key` 为空 → `ensure_active_provider()` 触发 `prompt_api_key()` TUI，填写后写回文件。

---

## 核心流程

### tai ask

```
AskArgs::handle
  ├─ TaiConfig::load()                    # 加载配置
  ├─ ensure_active_provider()             # 若 api_key 空 → TUI 引导
  └─ loop:
       do_ask(provider, model, prompt, config)
         ├─ Spinner + TextRenderer
         ├─ chat_stream() 流式回调
         └─ renderer.finish(config.show_markdown_view)
       Ok  → save_history() → return
       AuthError → recover_auth_error() → retry
       Err → return Err
```

### tai model

```
ModelArgs::handle
  ├─ subcommand=Config → handle_config()
  │    └─ config_providers(entries) TUI → save_providers()
  ├─ switch=Some(name) → switch_model()
  └─ 无参数 → select_model() TUI → save_active_model()
```

### tai config

```
ConfigCommand::handle
  ├─ TaiConfig::load()
  ├─ config_to_items(&config) → Vec<SettingItem>
  ├─ show_settings(items) TUI（alternate screen）
  └─ items_to_config() → new_config.save()
```

---

## 扩展指南

### 添加新命令
1. `lib.rs` Commands 枚举添加变体
2. 新建 `command_name.rs`
3. `lib.rs` handle() 添加分支

### 添加新 AI Provider
1. `tai-ai/src/provider.rs` 添加 `AiClient` 枚举变体
2. `tai-ai/src/lib.rs` 添加客户端单例 + `classify_error` + `chat_stream` 分支
3. `assets/providers.json` 添加默认条目（api_key 留空）

### 添加新 TUI 组件
- **非全屏**（小列表/输入）: 参考 `api_key_input.rs`，记录 `start_row`，输入行增量更新
- **全屏**（设置/查看）: 参考 `settings.rs` / `viewer.rs`，使用 alternate screen

### 添加新配置项
1. `tai-core/src/config.rs` TaiConfig 添加字段 + Default 值
2. `tai-command/src/config.rs` 的 `config_to_items` / `items_to_config` 添加对应项
3. 在相关命令中使用 `TaiConfig::load()`

### TUI 组件与 tai-ai 解耦
`tai-tui` 不依赖 `tai-ai`。需要传递 Provider 数据时，在 `tai-tui` 中定义镜像结构体（如 `ProviderEntry`），在 `tai-command` 中做转换。
