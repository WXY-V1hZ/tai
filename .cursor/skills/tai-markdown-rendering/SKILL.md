---
name: tai-markdown-rendering
description: Tai 项目 Markdown 渲染引擎开发和维护指南。在修改 reasoning.rs、viewer.rs 或处理 Markdown 渲染相关问题时使用。包括流式渲染策略、termimad 使用、alternate screen 管理。
---

# Tai Markdown 渲染引擎指南

## 核心设计理念

Tai 的 Markdown 渲染采用**两阶段策略**：

1. **流式阶段**: 原始 markdown 增量输出，避免闪烁
2. **完成阶段**: 使用 `termimad` 美化渲染，支持滚动查看

这种设计平衡了**实时性**和**展示效果**。

## 架构组件

### reasoning.rs - 流式渲染引擎

**位置**: `crates/tai-tui/src/reasoning.rs`

**职责**:
- 管理 reasoning 和 answer 的分离渲染
- 流式阶段的增量输出
- 完成阶段的美化展示

**关键结构**:

```rust
pub struct ReasoningRenderer {
    reasoning_buffer: String,  // 思考过程缓冲
    answer_buffer: String,     // 回答内容缓冲
}

impl ReasoningRenderer {
    pub fn new() -> Self;
    pub fn print_reasoning(&mut self, text: &str);
    pub fn print_answer(&mut self, text: &str);
    pub fn finish(self) -> Result<String>;
}
```

### viewer.rs - 公共展示模块

**位置**: `crates/tai-tui/src/viewer.rs`

**职责**:
- 提供通用的 markdown alternate screen 展示
- 处理键盘事件（滚动、退出）
- 避免代码重复

**导出函数**:

```rust
pub fn show_markdown_view(markdown: &str, skin: MadSkin) -> Result<()>;
pub fn make_default_skin() -> MadSkin;
```

## 流式阶段实现

### Reasoning 输出（灰色）

```rust
pub fn print_reasoning(&mut self, text: &str) {
    // 追加到缓冲区
    self.reasoning_buffer.push_str(text);
    
    // 设置灰色样式
    print!("{}", style(text).with(Color::DarkGrey));
    
    // 刷新输出
    let _ = std::io::stdout().flush();
}
```

**关键点**:
- 使用 `crossterm::style` 设置颜色
- 立即 `flush()` 确保实时显示
- 缓冲区保存完整内容供后续使用

### Answer 输出（原始）

```rust
pub fn print_answer(&mut self, text: &str) {
    // 追加到缓冲区
    self.answer_buffer.push_str(text);
    
    // 直接输出原始 markdown
    print!("{}", text);
    
    // 刷新输出
    let _ = std::io::stdout().flush();
}
```

**为什么不立即渲染 markdown**:
1. `termimad` 需要完整 markdown 才能正确渲染
2. 增量渲染会导致闪烁
3. 原始 markdown 仍然可读

## 完成阶段实现

### finish() 方法

```rust
pub fn finish(self) -> Result<String> {
    // 1. 打印分隔符
    println!("\n{}", "=".repeat(80));
    println!("按任意键查看详细内容...");
    
    // 2. 等待用户按键
    read()?;
    
    // 3. 进入 alternate screen 展示
    let skin = make_default_skin();
    show_markdown_view(&self.answer_buffer, skin)?;
    
    // 4. 只返回 answer（用于历史记录）
    Ok(self.answer_buffer)
}
```

**关键决策**:
- **只返回 answer**: 思考过程不保存，节省空间
- **等待用户**: 让用户决定何时查看详细内容
- **alternate screen**: 不影响终端历史记录

### show_markdown_view() 实现

```rust
pub fn show_markdown_view(markdown: &str, skin: MadSkin) -> Result<()> {
    let mut stdout = std::io::stdout();
    
    // 进入 alternate screen
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    
    // 创建 MadView
    let area = terminal_size()?;
    let mut view = MadView::from(markdown, area, skin);
    
    // 事件循环
    loop {
        view.write_on(&mut stdout)?;
        stdout.flush()?;
        
        match read()? {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        view.try_scroll_lines(-1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        view.try_scroll_lines(1);
                    }
                    KeyCode::PageUp => {
                        view.try_scroll_pages(-1);
                    }
                    KeyCode::PageDown => {
                        view.try_scroll_pages(1);
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
            Event::Resize(width, height) => {
                view.resize(&Area::new(0, 0, width, height));
            }
            _ => {}
        }
    }
    
    // 退出 alternate screen
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    
    Ok(())
}
```

## Termimad 配置

### 创建默认皮肤

```rust
pub fn make_default_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    
    // 代码块样式
    skin.code_block.set_fg(Color::Rgb { r: 200, g: 200, b: 200 });
    skin.code_block.set_bg(Color::Rgb { r: 40, g: 40, b: 40 });
    
    // 行内代码样式
    skin.inline_code.set_fg(Color::Rgb { r: 255, g: 200, b: 100 });
    
    // 标题样式
    skin.headers[0].set_fg(Color::Cyan);
    skin.headers[1].set_fg(Color::Green);
    
    // 表格边框
    skin.table.set_fg(Color::White);
    
    skin
}
```

**支持的元素**:
- 标题（H1-H6）
- 代码块（带语法高亮语言提示）
- 行内代码
- 表格
- 列表（有序、无序、嵌套）
- 粗体、斜体
- 引用

## 已知问题和解决方案

### 1. ANSI 转义序列污染

**问题**: AI 输出的 chunk 包含 ANSI 标记（如 `[0m`）

**位置**: `reasoning.rs:52-63`

**解决方案**:

```rust
fn strip_ansi(s: &str) -> String {
    // 方案1: 正则表达式
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    re.replace_all(s, "").to_string()
    
    // 方案2: strip-ansi-escapes crate
    String::from_utf8_lossy(&strip_ansi_escapes::strip(s)).to_string()
}

pub fn print_reasoning(&mut self, text: &str) {
    let clean_text = strip_ansi(text);
    self.reasoning_buffer.push_str(&clean_text);
    // ...
}
```

### 2. 代码块格式问题

**问题**: `termimad` 渲染代码块时有额外标记

**测试文件**: `test_response.md`

**调试方法**:

```rust
// 打印原始 markdown 查看问题
eprintln!("Raw markdown:\n{}", markdown);

// 测试简单代码块
let test_md = r#"
```rust
fn main() {
    println!("Hello");
}
```
"#;
show_markdown_view(test_md, make_default_skin())?;
```

**可能原因**:
- `termimad` 版本问题（当前 0.31）
- 代码块前后的空行处理
- 语言标识符解析

### 3. 表格渲染

**当前状态**: ✅ 正常工作

**注意事项**:
- 表格需要完整 markdown 才能正确对齐
- 不适合流式渲染
- 在 finish() 阶段展示效果良好

### 4. 窗口大小变化

**处理**: 已在 `show_markdown_view()` 中监听 `Event::Resize`

```rust
Event::Resize(width, height) => {
    view.resize(&Area::new(0, 0, width, height));
}
```

## 使用场景

### 1. AI 回答展示

```rust
// 在 ask.rs 中
let mut renderer = ReasoningRenderer::new();

chat_stream(
    |reasoning| renderer.print_reasoning(reasoning),
    |answer| renderer.print_answer(answer),
).await?;

let answer = renderer.finish()?;
```

### 2. 历史记录查看

```rust
// 在 history.rs 中
let markdown = read_history(&entry.path)?;
let skin = make_default_skin();
show_markdown_view(&markdown, skin)?;
```

### 3. 其他 Markdown 内容展示

```rust
// 通用展示
let content = fs::read_to_string("document.md")?;
show_markdown_view(&content, make_default_skin())?;
```

## 性能优化

### 1. 避免频繁重渲染

```rust
// 不好：每次 chunk 都重新渲染整个 markdown
for chunk in chunks {
    render_full_markdown(&buffer);  // ❌ 闪烁
}

// 好：流式阶段只增量打印
for chunk in chunks {
    print!("{}", chunk);            // ✅ 稳定
    flush()?;
}
```

### 2. 缓冲区管理

```rust
// 预分配缓冲区
pub fn new() -> Self {
    Self {
        reasoning_buffer: String::with_capacity(4096),
        answer_buffer: String::with_capacity(4096),
    }
}
```

### 3. 减少 flush() 调用

```rust
// 可以批量累积再 flush
for chunk in small_chunks {
    buffer.push_str(chunk);
}
flush()?;  // 一次 flush
```

## 测试指南

### 测试文件

**位置**: `d:\program\proj\tai\test_response.md`

包含的测试元素:
- 各级标题
- 代码块（多种语言）
- 表格
- 列表（嵌套）
- 文本样式
- 引用

### 测试命令

```bash
# 测试完整渲染流程
tai ask test

# 测试历史记录查看
tai ask "测试问题"
tai ask -c
```

### 手动测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_markdown_render() {
        let markdown = r#"
# Title
- Item 1
- Item 2
```rust
fn test() {}
```
"#;
        let skin = make_default_skin();
        show_markdown_view(markdown, skin).unwrap();
    }
}
```

## 扩展指南

### 添加新样式

```rust
pub fn make_custom_skin() -> MadSkin {
    let mut skin = make_default_skin();
    
    // 添加链接样式
    skin.link.set_fg(Color::Blue);
    skin.link.add_attr(Attribute::Underlined);
    
    // 添加引用样式
    skin.quote.set_fg(Color::Yellow);
    
    skin
}
```

### 支持自定义语法高亮

```rust
// termimad 本身不支持完整语法高亮
// 可以考虑：
// 1. 使用 syntect crate 预处理代码块
// 2. 将高亮后的 ANSI 文本嵌入 markdown
// 3. 或使用外部工具（如 bat）
```

### 添加新快捷键

```rust
match key_event.code {
    KeyCode::Char('g') => {
        view.try_scroll_lines(-10000);  // 跳到顶部
    }
    KeyCode::Char('G') => {
        view.try_scroll_lines(10000);   // 跳到底部
    }
    // 添加更多快捷键
    _ => {}
}
```

## 常见问题

### Q: 为什么不使用其他渲染库？

A: 
- `termimad` 与 `crossterm` 集成良好
- 支持滚动查看
- Rust 原生实现，性能好
- 已满足当前需求

### Q: 可以支持彩色代码高亮吗？

A: 
- `termimad` 不内置语法高亮
- 可以集成 `syntect` 预处理
- 或考虑使用 `bat` 库的核心组件

### Q: 如何处理超长行？

A: 
- `termimad` 会自动换行
- 或在皮肤中设置 `wrap_width`

```rust
skin.code_block.wrap = WrapMode::WordWrap;
```

### Q: 如何添加搜索功能？

A: 
- `MadView` 不内置搜索
- 需要自己实现：
  1. 接收搜索输入
  2. 在 markdown 中查找位置
  3. 调用 `view.try_scroll_lines()` 跳转

## 参考资源

- [termimad GitHub](https://github.com/Canop/termimad)
- [crossterm 文档](https://docs.rs/crossterm/)
- [CommonMark Spec](https://commonmark.org/) - Markdown 标准
- `markdown_render.md` - 项目内设计文档
