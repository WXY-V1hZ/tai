use std::fs::{self, File};
use std::io::{self, Write as IoWrite};
use std::path::PathBuf;
use tai_core::{TaiError, TaiResult};
use tai_tui::{show_markdown_view, make_default_skin};
use tracing::{debug, warn};
use chrono::Local;
use termimad::{
    crossterm::{
        cursor::{Hide, Show},
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        queue,
        terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    },
};

const MAX_HISTORY_COUNT: usize = 50;

/// 获取历史记录目录路径
fn get_history_dir() -> TaiResult<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| TaiError::FileError("无法获取用户目录".to_string()))?;
    let history_dir = home.join(".tai").join("cache").join("history");
    
    if !history_dir.exists() {
        fs::create_dir_all(&history_dir)?;
        debug!("创建历史记录目录: {:?}", history_dir);
    }
    
    Ok(history_dir)
}

/// 保存一条历史记录
pub fn save_history(markdown: &str) -> TaiResult<()> {
    let history_dir = get_history_dir()?;
    
    // 生成文件名：timestamp.md
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}.md", timestamp);
    let filepath = history_dir.join(&filename);
    
    // 保存文件
    let mut file = File::create(&filepath)?;
    file.write_all(markdown.as_bytes())?;
    debug!("保存历史记录: {:?}", filepath);
    
    // 检查并清理旧记录
    cleanup_old_history(&history_dir)?;
    
    Ok(())
}

/// 清理超出数量限制的历史记录
fn cleanup_old_history(history_dir: &PathBuf) -> TaiResult<()> {
    let mut entries: Vec<_> = fs::read_dir(history_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
        .collect();
    
    if entries.len() <= MAX_HISTORY_COUNT {
        return Ok(());
    }
    
    // 按修改时间排序（最新的在前）
    entries.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });
    
    // 删除超出限制的旧文件
    for entry in entries.iter().skip(MAX_HISTORY_COUNT) {
        let path = entry.path();
        if let Err(e) = fs::remove_file(&path) {
            warn!("删除旧历史记录失败: {:?}, 错误: {}", path, e);
        } else {
            debug!("删除旧历史记录: {:?}", path);
        }
    }
    
    Ok(())
}

/// 获取历史记录列表（最新的在前）
fn list_history(count: usize) -> TaiResult<Vec<HistoryEntry>> {
    let history_dir = get_history_dir()?;
    
    let mut entries: Vec<_> = fs::read_dir(history_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let path = e.path();
            let metadata = e.metadata().ok()?;
            let modified = metadata.modified().ok()?;
            
            Some(HistoryEntry {
                path,
                modified,
            })
        })
        .collect();
    
    // 按修改时间排序（最新的在前）
    entries.sort_by(|a, b| b.modified.cmp(&a.modified));
    
    // 限制数量
    entries.truncate(count);
    
    Ok(entries)
}

/// 读取历史记录内容
fn read_history(path: &PathBuf) -> TaiResult<String> {
    fs::read_to_string(path).map_err(|e| TaiError::FileError(e.to_string()))
}

#[derive(Debug)]
struct HistoryEntry {
    path: PathBuf,
    modified: std::time::SystemTime,
}

/// 显示历史记录（使用 alternate screen）
pub fn show_history(count: usize) -> TaiResult<()> {
    let entries = list_history(count)?;
    
    if entries.is_empty() {
        println!("没有历史记录");
        return Ok(());
    }
    
    // 如果只有一条记录，直接显示
    if entries.len() == 1 {
        let content = read_history(&entries[0].path)?;
        return show_single_history(&content);
    }
    
    // 多条记录，显示选择菜单
    show_history_list(entries)
}

/// 显示单条历史记录
fn show_single_history(content: &str) -> TaiResult<()> {
    show_markdown_view(content, make_default_skin())
        .map_err(|e| TaiError::FileError(e.to_string()))
}

/// 显示历史记录列表并允许选择
fn show_history_list(entries: Vec<HistoryEntry>) -> TaiResult<()> {
    let mut w = io::stdout();
    let mut selected: usize = 0;
    
    terminal::enable_raw_mode().map_err(|e| TaiError::FileError(e.to_string()))?;
    queue!(w, EnterAlternateScreen, Hide).map_err(|e| TaiError::FileError(e.to_string()))?;
    w.flush().map_err(|e| TaiError::FileError(e.to_string()))?;
    
    flush_pending_events();
    
    loop {
        // 渲染列表
        queue!(w, Clear(ClearType::All)).map_err(|e| TaiError::FileError(e.to_string()))?;
        queue!(w, termimad::crossterm::cursor::MoveTo(0, 0)).map_err(|e| TaiError::FileError(e.to_string()))?;
        
        writeln!(w, "历史记录 (共 {} 条)", entries.len()).map_err(|e| TaiError::FileError(e.to_string()))?;
        writeln!(w, "使用 ↑↓ 选择，回车查看，ESC 退出\n").map_err(|e| TaiError::FileError(e.to_string()))?;
        
        for (i, entry) in entries.iter().enumerate() {
            let prefix = if i == selected { "→ " } else { "  " };
            let time = format_time(entry.modified);
            writeln!(w, "{}[{}] {}", prefix, i + 1, time).map_err(|e| TaiError::FileError(e.to_string()))?;
        }
        
        w.flush().map_err(|e| TaiError::FileError(e.to_string()))?;
        
        // 处理按键
        match event::read() {
            Ok(Event::Key(KeyEvent { code, modifiers, kind: KeyEventKind::Press, .. })) => {
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                match code {
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected < entries.len() - 1 {
                            selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        // 退出列表，显示选中的历史记录
                        terminal::disable_raw_mode().map_err(|e| TaiError::FileError(e.to_string()))?;
                        queue!(w, Show, LeaveAlternateScreen).map_err(|e| TaiError::FileError(e.to_string()))?;
                        w.flush().map_err(|e| TaiError::FileError(e.to_string()))?;
                        
                        let content = read_history(&entries[selected].path)?;
                        return show_single_history(&content);
                    }
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
            Ok(Event::Key(_)) => {}
            Ok(Event::Resize(..)) => {}
            _ => {}
        }
    }
    
    terminal::disable_raw_mode().map_err(|e| TaiError::FileError(e.to_string()))?;
    queue!(w, Show, LeaveAlternateScreen).map_err(|e| TaiError::FileError(e.to_string()))?;
    w.flush().map_err(|e| TaiError::FileError(e.to_string()))?;
    
    Ok(())
}

fn format_time(time: std::time::SystemTime) -> String {
    use chrono::{DateTime, Utc};
    let datetime: DateTime<Utc> = time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn flush_pending_events() {
    while event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) {
        let _ = event::read();
    }
}
