# Tai

AI 驱动的命令行助手，支持多厂商 AI 模型。

## 功能

- 🤖 **智能命令生成** - 自然语言描述转命令行
- 💬 **AI 对话** - 支持流式输出和推理过程可视化
- 🔄 **多模型支持** - OpenAI、DeepSeek 等多厂商模型
- 🎨 **友好的错误提示** - 清晰的错误信息和解决建议

## 安装

目前仅支持编译安装

```bash
cargo build --release
```

可执行文件位于 `target/release`

## 配置

在 `~/.tai/providers.json` 配置 AI 提供商：

```json
[
  {
    "provider": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "your-api-key",
    "model_names": ["gpt-4o-mini", "gpt-4o"]
  },
  {
    "provider": "deepseek",
    "base_url": "https://api.deepseek.com",
    "api_key": "your-api-key",
    "model_names": ["deepseek-chat", "deepseek-reasoner"]
  }
]
```

## 使用

### 模型管理

```bash
# 查看可用模型
tai model

# 切换模型
tai model gpt-4o-mini
```

### 智能命令生成

```bash
# 生成命令并自动复制到剪贴板
tai go "列出当前目录下所有 .rs 文件"
```

### AI 对话

```bash
# 流式对话（支持推理过程可视化）
tai ask "解释 Rust 的所有权机制"

# 附加文件
tai ask -f config.toml "解释这个配置文件"
```

### 系统初始化

```bash
# 收集系统信息到 ~/.tai/sysinfo.txt
tai init
```
