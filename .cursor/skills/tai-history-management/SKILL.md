---
name: tai-history-management
description: Tai 项目历史记录功能开发和维护指南。在修改 history.rs、处理历史记录保存/查看/清理逻辑时使用。包括文件命名规则、自动清理策略、交互式查看实现。
---

# Tai 历史记录管理指南

## 功能概述

**实现时间**: 2026-02-20  
**位置**: `crates/tai-command/src/ask/history.rs`

**核心功能**:
1. 自动保存 AI 回答到本地
2. 查看历史记录列表
3. 交互式选择和查看
4. 自动清理超出限制的记录

## 文件结构

### 存储位置

```
~/.tai/
└── cache/
    └── history/
        ├── 20260220_091234.md
        ├── 20260220_102345.md
        └── 20260220_145623.md
```

### 文件命名规则

格式: `YYYYMMDD_HHMMSS.md`

```rust
use chrono::Local;

let timestamp = Local::now().format("%Y%m%d_%H%M%S");
let filename = format!("{}.md", timestamp);
```

**优点**:
- 按时间排序
- 文件名即时间戳
- 避免命名冲突

### 配置常量

```rust
const MAX_HISTORY_COUNT: usize = 50;  // 最多保存 50 条
```

## 核心函数

### save_history() - 保存历史

```rust
pub fn save_history(markdown: &str) -> Result<()> {
    // 1. 获取历史目录
    let history_dir = get_history_dir()?;
    fs::create_dir_all(&history_dir)?;
    
    // 2. 生成文件名
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}.md", timestamp);
    let file_path = history_dir.join(filename);
    
    // 3. 写入文件
    let mut file = BufWriter::new(File::create(&file_path)?);
    file.write_all(markdown.as_bytes())?;
    file.flush()?;
    
    tracing::info!("Saved history to {:?}", file_path);
    
    // 4. 自动清理
    cleanup_old_history()?;
    
    Ok(())
}
```

**使用场景**: 在 `ask.rs` 中调用

```rust
let answer = renderer.finish()?;

// 保存失败不影响主流程
if let Err(e) = save_history(&answer) {
    tracing::warn!("Failed to save history: {}", e);
}
```

### show_history() - 显示历史

```rust
pub fn show_history(count: usize) -> Result<()> {
    // 1. 获取历史列表
    let entries = list_history(count)?;
    
    if entries.is_empty() {
        println!("没有历史记录。");
        return Ok(());
    }
    
    // 2. 单条记录直接显示
    if entries.len() == 1 {
        let markdown = read_history(&entries[0].path)?;
        let skin = make_default_skin();
        show_markdown_view(&markdown, skin)?;
        return Ok(());
    }
    
    // 3. 多条记录交互式选择
    let selected = select_history_interactive(&entries)?;
    let markdown = read_history(&entries[selected].path)?;
    let skin = make_default_skin();
    show_markdown_view(&markdown, skin)?;
    
    Ok(())
}
```

**命令行使用**:

```bash
tai ask -c       # 查看上一次（默认 1 条）
tai ask -c 10    # 查看最近 10 条
```

### list_history() - 列出历史

```rust
pub fn list_history(count: usize) -> Result<Vec<HistoryEntry>> {
    let history_dir = get_history_dir()?;
    
    // 1. 读取目录
    let mut entries: Vec<HistoryEntry> = Vec::new();
    for entry in fs::read_dir(&history_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        // 只处理 .md 文件
        if !path.extension().map_or(false, |ext| ext == "md") {
            continue;
        }
        
        // 获取修改时间
        let metadata = fs::metadata(&path)?;
        let modified = metadata.modified()?;
        
        entries.push(HistoryEntry {
            path,
            modified,
            filename: entry.file_name().to_string_lossy().to_string(),
        });
    }
    
    // 2. 按修改时间倒序排序（最新在前）
    entries.sort_by(|a, b| b.modified.cmp(&a.modified));
    
    // 3. 只返回前 count 条
    entries.truncate(count);
    
    Ok(entries)
}
```

### cleanup_old_history() - 清理旧记录

```rust
fn cleanup_old_history() -> Result<()> {
    let history_dir = get_history_dir()?;
    
    // 1. 获取所有历史记录
    let mut entries: Vec<HistoryEntry> = Vec::new();
    for entry in fs::read_dir(&history_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if !path.extension().map_or(false, |ext| ext == "md") {
            continue;
        }
        
        let metadata = fs::metadata(&path)?;
        entries.push(HistoryEntry {
            path,
            modified: metadata.modified()?,
            filename: entry.file_name().to_string_lossy().to_string(),
        });
    }
    
    // 2. 如果超出限制，删除最旧的
    if entries.len() <= MAX_HISTORY_COUNT {
        return Ok(());
    }
    
    // 按修改时间排序
    entries.sort_by(|a, b| b.modified.cmp(&a.modified));
    
    // 删除超出的部分
    for entry in entries.iter().skip(MAX_HISTORY_COUNT) {
        fs::remove_file(&entry.path)?;
        tracing::info!("Deleted old history: {:?}", entry.path);
    }
    
    Ok(())
}
```

**触发时机**: 每次 `save_history()` 后自动调用

### read_history() - 读取历史内容

```rust
fn read_history(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}
```

## 交互式选择实现

### select_history_interactive()

```rust
fn select_history_interactive(entries: &[HistoryEntry]) -> Result<usize> {
    let mut stdout = std::io::stdout();
    
    // 进入 alternate screen
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    
    let mut selected = 0;
    
    loop {
        // 清屏
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
        
        // 显示标题
        println!("历史记录列表（共 {} 条）：\n", entries.len());
        
        // 显示列表
        for (i, entry) in entries.iter().enumerate() {
            // 提取时间戳
            let display_name = extract_timestamp(&entry.filename);
            
            if i == selected {
                println!("  > {} <", style(display_name).cyan());
            } else {
                println!("    {}", display_name);
            }
        }
        
        println!("\n使用上下键选择，Enter 确认，q 退出");
        stdout.flush()?;
        
        // 处理按键
        match read()? {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if selected < entries.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        break;
                    }
                    KeyCode::Char('q') | KeyCode::Esc => {
                        disable_raw_mode()?;
                        execute!(stdout, LeaveAlternateScreen)?;
                        return Err(anyhow!("用户取消"));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    
    // 退出 alternate screen
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    
    Ok(selected)
}
```

**交互逻辑**:
1. 进入 alternate screen
2. 显示列表，高亮当前选中项
3. 上下键切换选择
4. Enter 确认，q 退出
5. 返回选中索引

### extract_timestamp() - 格式化时间戳

```rust
fn extract_timestamp(filename: &str) -> String {
    // 从 "20260220_091234.md" 提取并格式化
    if let Some(stem) = filename.strip_suffix(".md") {
        if let Some((date, time)) = stem.split_once('_') {
            // 格式化为 "2026-02-20 09:12:34"
            if date.len() == 8 && time.len() == 6 {
                let year = &date[0..4];
                let month = &date[4..6];
                let day = &date[6..8];
                let hour = &time[0..2];
                let minute = &time[2..4];
                let second = &time[4..6];
                
                return format!(
                    "{}-{}-{} {}:{}:{}",
                    year, month, day, hour, minute, second
                );
            }
        }
    }
    
    // 回退到原始文件名
    filename.to_string()
}
```

## 数据结构

```rust
struct HistoryEntry {
    path: PathBuf,           // 文件路径
    modified: SystemTime,    // 修改时间
    filename: String,        // 文件名
}
```

## 集成到命令行

### 添加 -c 参数

```rust
// crates/tai-command/src/lib.rs
#[derive(Parser, Debug)]
pub struct AskArgs {
    // ... 其他参数 ...
    
    /// 查看历史记录缓存（可选指定数量，默认为 1）
    #[arg(short, long, num_args = 0..=1, default_missing_value = "1")]
    pub cache: Option<usize>,
}
```

**参数行为**:
- `tai ask -c` → `cache = Some(1)` （默认）
- `tai ask -c 10` → `cache = Some(10)`
- `tai ask` → `cache = None`

### 命令处理逻辑

```rust
// crates/tai-command/src/ask.rs
pub async fn handle_ask(args: AskArgs) -> Result<()> {
    // 如果指定了 -c，显示历史
    if let Some(count) = args.cache {
        return show_history(count);
    }
    
    // 否则正常提问
    // ...
    
    // 提问完成后保存历史
    if let Err(e) = save_history(&answer) {
        tracing::warn!("Failed to save history: {}", e);
    }
    
    Ok(())
}
```

## 错误处理策略

### 保存失败不中断

```rust
// 在 ask.rs 中
if let Err(e) = save_history(&answer) {
    tracing::warn!("Failed to save history: {}", e);
    // 不返回错误，继续执行
}
```

**原因**: 历史记录是辅助功能，不应影响主流程

### 查看失败提示用户

```rust
pub fn show_history(count: usize) -> Result<()> {
    let entries = list_history(count)
        .context("无法读取历史记录目录")?;
    
    if entries.is_empty() {
        println!("没有历史记录。");
        return Ok(());
    }
    
    // ...
}
```

## 性能优化

### 使用 BufWriter

```rust
let file = File::create(&file_path)?;
let mut writer = BufWriter::new(file);
writer.write_all(markdown.as_bytes())?;
writer.flush()?;
```

### 限制读取数量

```rust
// 不读取所有文件，只读取需要的数量
entries.truncate(count);
```

### 延迟加载内容

```rust
// 列表阶段只读取元数据
for entry in fs::read_dir(&history_dir)? {
    let metadata = fs::metadata(&path)?;
    // 不读取文件内容
}

// 选中后才读取内容
let markdown = read_history(&selected_path)?;
```

## 扩展功能建议

### 1. 搜索功能

```rust
pub fn search_history(keyword: &str) -> Result<Vec<HistoryEntry>> {
    let all_entries = list_history(usize::MAX)?;
    
    let mut results = Vec::new();
    for entry in all_entries {
        let content = read_history(&entry.path)?;
        if content.contains(keyword) {
            results.push(entry);
        }
    }
    
    Ok(results)
}
```

**使用**: `tai ask -s "Rust"`

### 2. 时间范围筛选

```rust
pub fn list_history_by_date_range(
    start: DateTime<Local>,
    end: DateTime<Local>,
) -> Result<Vec<HistoryEntry>> {
    let all_entries = list_history(usize::MAX)?;
    
    Ok(all_entries
        .into_iter()
        .filter(|e| {
            let modified = DateTime::<Local>::from(e.modified);
            modified >= start && modified <= end
        })
        .collect())
}
```

**使用**: `tai ask --from "2026-02-20" --to "2026-02-21"`

### 3. 导出功能

```rust
pub fn export_history(output: &Path, count: usize) -> Result<()> {
    let entries = list_history(count)?;
    
    let mut output_file = BufWriter::new(File::create(output)?);
    
    for entry in entries {
        let content = read_history(&entry.path)?;
        writeln!(output_file, "# {}\n", entry.filename)?;
        writeln!(output_file, "{}\n", content)?;
        writeln!(output_file, "---\n")?;
    }
    
    output_file.flush()?;
    Ok(())
}
```

**使用**: `tai ask --export history.md -c 10`

### 4. 删除功能

```rust
pub fn delete_history(index: usize) -> Result<()> {
    let entries = list_history(usize::MAX)?;
    
    if index >= entries.len() {
        return Err(anyhow!("索引超出范围"));
    }
    
    fs::remove_file(&entries[index].path)?;
    println!("已删除历史记录：{}", entries[index].filename);
    
    Ok(())
}
```

**使用**: `tai ask --delete 0` （删除最新的）

### 5. 标签功能

**方案**: 在文件头添加 YAML frontmatter

```markdown
---
tags: [rust, async, tokio]
title: "如何使用 Tokio"
---

# 回答内容
...
```

**实现**: 修改 `save_history()` 支持添加元数据

## 测试指南

### 功能测试

```bash
# 1. 保存历史（自动）
tai ask "测试问题1"
tai ask "测试问题2"
tai ask "测试问题3"

# 2. 查看上一次
tai ask -c

# 3. 查看多条历史
tai ask -c 10

# 4. 检查文件
ls ~/.tai/cache/history/

# 5. 测试自动清理
# 创建 51 条记录，检查是否保留 50 条
```

### 边界测试

```bash
# 空历史
rm -rf ~/.tai/cache/history/
tai ask -c

# 单条历史
tai ask "test"
tai ask -c

# 超出限制（> 50 条）
# 测试自动清理
```

### 错误测试

```bash
# 权限问题
chmod 000 ~/.tai/cache/history/
tai ask "test"  # 应记录日志但不崩溃

# 磁盘空间不足
# 模拟测试
```

## 日志记录

```rust
// 成功保存
tracing::info!("Saved history to {:?}", file_path);

// 清理旧记录
tracing::info!("Deleted old history: {:?}", entry.path);

// 保存失败（在 ask.rs）
tracing::warn!("Failed to save history: {}", e);

// 清理失败
tracing::warn!("Failed to cleanup old history: {}", e);
```

## 常见问题

### Q: 为什么不保存 reasoning？

A: 
- reasoning 通常很长，占用空间
- 用户主要关心 answer 部分
- 可以节省存储空间
- 如需保存，修改 `ask.rs` 传递完整内容

### Q: 如何修改最大保存数量？

A: 
修改 `history.rs` 中的常量：

```rust
const MAX_HISTORY_COUNT: usize = 100;  // 改为 100
```

### Q: 可以跨设备同步吗？

A: 
当前不支持，建议方案：
1. 使用云盘同步 `~/.tai/cache/history/`
2. 实现云端存储（需要添加后端）
3. Git 仓库管理历史记录

### Q: 如何备份历史记录？

A: 

```bash
# 手动备份
cp -r ~/.tai/cache/history/ ~/backup/tai-history/

# 或使用导出功能（待实现）
tai ask --export all-history.md -c 1000
```

## 参考资源

- `HISTORY_FEATURE.md` - 功能需求文档
- `viewer.rs` - MadView 展示模块
- `reasoning.rs` - finish() 返回值说明
