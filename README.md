**English** | **[简体中文](README.zh-CN.md)**

# Tai (态)

An AI-driven command-line assistant with streaming output, markdown rendering, and multi-provider support.

Tai is a Rust-based CLI tool that brings conversational AI to your terminal, with:
- Beautiful streaming output with reasoning process visualization
- Native markdown rendering with syntax highlighting
- Automatic conversation history management
- Multi-provider support (OpenAI, DeepSeek, and more)
- Fast and efficient with zero active loops

## Key Features

### 🤖 AI Conversation (`tai ask`)

Stream responses from AI models directly in your terminal, with real-time reasoning process visualization:

```bash
tai ask "Explain Rust's ownership system"
```

![reasoning-demo](doc/reasoning-demo.gif)

**Features:**
- **Streaming output**: See the AI's response as it's generated
- **Reasoning visualization**: Watch the thinking process in gray text
- **Markdown rendering**: Tables, code blocks, and formatting rendered beautifully
- **Scrollable view**: Navigate long responses with arrow keys
- **File attachment**: Include files as context with `-f`

### 📜 Conversation History (`tai ask -c`)

Automatically saves every conversation and lets you revisit them anytime:

```bash
# View last conversation
tai ask -c

# Browse recent 10 conversations
tai ask -c 10
```

**Features:**
- Automatic saving (up to 50 most recent)
- Interactive selection with arrow keys
- Full markdown rendering for history
- Smart cleanup of old entries

### 🎛️ Model Management (`tai model`)

Easily switch between different AI models and providers:

```bash
# Interactive model selector
tai model

# Direct switch
tai model gpt-4o-mini
```

![model-selector-demo](doc/model-selector-demo.gif)

Supported providers:
- **OpenAI**: GPT-4o, GPT-4o-mini
- **DeepSeek**: DeepSeek-Chat, DeepSeek-Reasoner
- Custom providers (via API compatibility)

### ⚡ Command Generation (`tai go`)

Natural language to command line conversion:

```bash
tai go "list all rust files"
# Generates and copies to clipboard: ls **/*.rs
```

### 🔧 System Info (`tai init`)

Collect system information for AI context:

```bash
tai init
```

Gathers OS info, environment details, and saves to `~/.tai/sysinfo.txt`.

## Installation

### From Source

```bash
git clone https://github.com/yourusername/tai.git
cd tai
cargo build --release
```

The binary will be in `target/release/tai`.

### System Requirements

- Rust 1.93 or later
- Windows 10+ / macOS / Linux
- Terminal with ANSI color support

## Configuration

### Provider Setup

Create `~/.tai/providers.json` with your API credentials:

```json
[
  {
    "provider": "openai",
    "base_url": "https://api.openai.com/v1",
    "api_key": "sk-your-api-key-here",
    "model_names": ["gpt-4o-mini", "gpt-4o"]
  },
  {
    "provider": "deepseek",
    "base_url": "https://api.deepseek.com",
    "api_key": "sk-your-api-key-here",
    "model_names": ["deepseek-chat", "deepseek-reasoner"]
  }
]
```

### Active Model

The current model is stored in `~/.tai/active_model.txt` as `provider/model_name`.

### File Structure

```
~/.tai/
├── providers.json          # API configuration
├── active_model.txt        # Current active model
├── sysinfo.txt            # System information
├── cache/
│   └── history/           # Conversation history
│       ├── 20260220_091234.md
│       └── ...
└── tai-*.log              # Rotating log files
```

## Usage Examples

### Basic Conversation

```bash
tai ask "What are the benefits of Rust?"
```

The response streams in real-time, with the reasoning process shown in gray and the answer in white.

### Conversation with Context

```bash
tai ask -f Cargo.toml "Analyze this project's dependencies"
```

Attaches `Cargo.toml` as context for the AI.

### Review History

```bash
# Last conversation
tai ask -c

# Browse recent conversations
tai ask -c 10
```

Use arrow keys to navigate, Enter to select, `q` to quit.

### Viewing Rendered Output

After streaming completes, press any key to enter the scrollable viewer:

![scrollable-viewer](doc/scrollable-viewer.gif)

**Controls:**
- `↑/k` - Scroll up
- `↓/j` - Scroll down  
- `PageUp` - Page up
- `PageDown` - Page down
- `q` - Exit viewer

## Markdown Rendering

Tai uses a two-stage rendering strategy for optimal performance:

1. **Streaming Stage**: Raw markdown output for real-time feedback
2. **Completion Stage**: Beautiful rendering with `termimad` in an alternate screen

### Supported Markdown

| Feature | Support | Example |
|---------|---------|---------|
| Headers (H1-H6) | ✅ | `# Title` |
| Code blocks | ✅ | ` ```rust\nfn main() {}\n``` ` |
| Inline code | ✅ | `` `code` `` |
| Tables | ✅ | `\| col1 \| col2 \|` |
| Lists | ✅ | `- item` / `1. item` |
| Bold/Italic | ✅ | `**bold**` / `*italic*` |
| Strikethrough | ✅ | `~~text~~` |
| Quotes | ✅ | `> quote` |
| Horizontal rules | ✅ | `---` |

## Architecture

Tai is built with a modular Rust workspace structure:

```
tai/
├── tai/                    # Main binary
├── crates/
│   ├── tai-command/        # CLI parsing and handlers
│   ├── tai-ai/            # AI client core
│   ├── tai-tui/           # Terminal UI components
│   ├── tai-core/          # Shared utilities
│   └── tai-pty/           # PTY support (WIP)
```

### Key Dependencies

- [rig-core](https://github.com/0xPlaygrounds/rig) - AI client framework
- [clap](https://github.com/clap-rs/clap) - Command line parsing
- [tokio](https://tokio.rs) - Async runtime
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal control
- [termimad](https://github.com/Canop/termimad) - Markdown rendering
- [tracing](https://github.com/tokio-rs/tracing) - Logging

## Performance

- **Zero active loops**: Event-driven architecture with no polling
- **Streaming**: Responses appear as they're generated, no waiting
- **Smart buffering**: Efficient incremental rendering
- **Singleton clients**: AI clients initialized once and reused
- **Minimal allocations**: Careful string handling and buffer reuse

## Development

### Building

```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test
```

### Running Examples

```bash
# Test mode (uses local test file)
tai ask test

# Scrollable view demo
cargo run --example scrollable
```

### Project Guidelines

- Maximum 500 lines per file (extract to submodules when exceeded)
- Use `tracing` for logging (debug to file, info to terminal)
- Prefer code reuse over duplication (see `viewer.rs`)
- Error handling: propagate in core functions, log in auxiliary functions
- See `.cursor/skills/tai-rust-best-practices/SKILL.md` for detailed guidelines

## Logging

Logs are written to `~/.tai/tai-{timestamp}.log` with:
- Hourly rotation
- Maximum 10 files kept
- Debug level in files, info level in terminal

View logs:

```bash
# Windows PowerShell
Get-Content ~\.tai\tai-*.log -Tail 50

# Unix
tail -f ~/.tai/tai-*.log
```

## Troubleshooting

### "No providers configured"

Create `~/.tai/providers.json` with your API credentials (see Configuration section).

### "Model not found"

Run `tai model` to see available models and select one.

### Streaming seems slow

Check your network connection. The AI provider's response time varies by model and load.

### Colors not showing correctly

Ensure your terminal supports ANSI colors. Windows users should use Windows Terminal or PowerShell 7+.

## Roadmap

- [ ] More AI providers (Anthropic, Google, Ollama)
- [ ] History search and filtering
- [ ] Export conversations to markdown
- [ ] Configuration via CLI commands
- [ ] Plugin system for custom commands
- [ ] PTY support for command execution
- [ ] Cross-platform path handling

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

### Guidelines

1. Follow the project's coding style (see skills documentation)
2. Add tests for new features
3. Update documentation as needed
4. Keep files under 500 lines
5. Use meaningful commit messages

## Related Projects

- [termimad](https://github.com/Canop/termimad) - The markdown rendering engine we use
- [rig](https://github.com/0xPlaygrounds/rig) - The AI client framework powering Tai

## Author

Created with ❤️ by V1hZ

---

**Note**: This is an early version of Tai. APIs and features may change as the project evolves. Feedback and contributions are highly appreciated!
