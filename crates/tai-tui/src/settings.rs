use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    QueueableCommand,
};
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

// ── Value types ───────────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum SettingValue {
    Bool(bool),
    Select { options: Vec<String>, selected: usize },
    Int { value: i64, min: i64, max: i64 },
}

#[derive(Clone)]
pub struct SettingItem {
    pub key: String,
    pub label: String,
    pub value: SettingValue,
}

impl SettingItem {
    pub fn bool(key: impl Into<String>, label: impl Into<String>, value: bool) -> Self {
        Self { key: key.into(), label: label.into(), value: SettingValue::Bool(value) }
    }

    pub fn select(
        key: impl Into<String>,
        label: impl Into<String>,
        options: Vec<String>,
        selected: usize,
    ) -> Self {
        Self { key: key.into(), label: label.into(), value: SettingValue::Select { options, selected } }
    }

    pub fn int(
        key: impl Into<String>,
        label: impl Into<String>,
        value: i64,
        min: i64,
        max: i64,
    ) -> Self {
        Self { key: key.into(), label: label.into(), value: SettingValue::Int { value, min, max } }
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// 在 alternate screen 中展示设置列表
/// 返回 Some(updated) 若用户按 s 保存，None 若取消
pub fn show_settings(mut items: Vec<SettingItem>) -> io::Result<Option<Vec<SettingItem>>> {
    if items.is_empty() {
        return Ok(None);
    }

    // raw mode 必须在 EnterAlternateScreen 前启用（Windows 兼容性要求）
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    flush_pending_events();

    let saved = settings_loop(&mut stdout, &mut items)?;

    let _ = execute!(stdout, LeaveAlternateScreen, cursor::Show);
    let _ = terminal::disable_raw_mode();

    Ok(if saved { Some(items) } else { None })
}

// ── Event loop ────────────────────────────────────────────────────────────────

fn settings_loop(stdout: &mut impl Write, items: &mut Vec<SettingItem>) -> io::Result<bool> {
    let mut selected = 0usize;
    let label_col = max_label_width(items);

    render(stdout, items, selected, label_col)?;

    loop {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Up => {
                    selected = if selected > 0 { selected - 1 } else { items.len() - 1 };
                    render(stdout, items, selected, label_col)?;
                }
                KeyCode::Down => {
                    selected = if selected < items.len() - 1 { selected + 1 } else { 0 };
                    render(stdout, items, selected, label_col)?;
                }
                KeyCode::Char(' ') | KeyCode::Enter => {
                    activate(&mut items[selected]);
                    render(stdout, items, selected, label_col)?;
                }
                KeyCode::Left => {
                    adjust(&mut items[selected], -1);
                    render(stdout, items, selected, label_col)?;
                }
                KeyCode::Right => {
                    adjust(&mut items[selected], 1);
                    render(stdout, items, selected, label_col)?;
                }
                KeyCode::Char('s') => return Ok(true),
                KeyCode::Esc | KeyCode::Char('q') => return Ok(false),
                _ => {}
            },
            Event::Resize(..) => render(stdout, items, selected, label_col)?,
            _ => {}
        }
    }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render(
    stdout: &mut impl Write,
    items: &[SettingItem],
    selected: usize,
    label_col: usize,
) -> io::Result<()> {
    stdout.queue(cursor::MoveTo(0, 0))?;
    stdout.queue(terminal::Clear(ClearType::All))?;

    stdout
        .queue(SetForegroundColor(Color::DarkGrey))?
        .queue(Print("  应用配置\n\n"))?
        .queue(ResetColor)?;

    for (i, item) in items.iter().enumerate() {
        render_item(stdout, item, i == selected, label_col)?;
    }

    let hint = footer_hint(&items[selected].value);
    stdout
        .queue(Print("\n"))?
        .queue(SetForegroundColor(Color::DarkGrey))?
        .queue(Print(hint))?
        .queue(Print("\n"))?
        .queue(ResetColor)?;

    stdout.flush()?;
    Ok(())
}

fn render_item(
    stdout: &mut impl Write,
    item: &SettingItem,
    is_selected: bool,
    label_col: usize,
) -> io::Result<()> {
    let prefix = if is_selected { "> " } else { "  " };
    // unicode-aware padding to align value column
    let pad = " ".repeat(label_col - item.label.width());

    if is_selected {
        stdout
            .queue(SetForegroundColor(Color::Cyan))?
            .queue(SetAttribute(Attribute::Bold))?
            .queue(Print(format!("{}  {}", prefix, item.label)))?
            .queue(SetAttribute(Attribute::Reset))?
            .queue(ResetColor)?;
    } else {
        stdout.queue(Print(format!("{}  {}", prefix, item.label)))?;
    }

    stdout.queue(Print(format!("{}  ", pad)))?;
    render_value(stdout, &item.value)?;
    stdout.queue(Print("\n"))?;

    Ok(())
}

fn render_value(stdout: &mut impl Write, value: &SettingValue) -> io::Result<()> {
    match value {
        SettingValue::Bool(true) => {
            stdout
                .queue(SetForegroundColor(Color::Green))?
                .queue(Print("●  开启"))?
                .queue(ResetColor)?;
        }
        SettingValue::Bool(false) => {
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print("○  关闭"))?
                .queue(ResetColor)?;
        }
        SettingValue::Select { options, selected } => {
            stdout
                .queue(SetForegroundColor(Color::Cyan))?
                .queue(Print(format!("◈  {}", options[*selected])))?
                .queue(ResetColor)?;
        }
        SettingValue::Int { value, .. } => {
            stdout
                .queue(SetForegroundColor(Color::Cyan))?
                .queue(Print(format!("‹ {} ›", value)))?
                .queue(ResetColor)?;
        }
    }
    Ok(())
}

fn footer_hint(value: &SettingValue) -> &'static str {
    match value {
        SettingValue::Bool(_) | SettingValue::Select { .. } => {
            "  Space 切换  ·  ↑↓ 导航  ·  s 保存  ·  Esc 退出"
        }
        SettingValue::Int { .. } => "  ← → 调整  ·  ↑↓ 导航  ·  s 保存  ·  Esc 退出",
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn max_label_width(items: &[SettingItem]) -> usize {
    items.iter().map(|i| i.label.width()).max().unwrap_or(0)
}

fn activate(item: &mut SettingItem) {
    match &mut item.value {
        SettingValue::Bool(v) => *v = !*v,
        SettingValue::Select { options, selected } => {
            *selected = (*selected + 1) % options.len();
        }
        SettingValue::Int { .. } => {}
    }
}

fn adjust(item: &mut SettingItem, delta: i64) {
    if let SettingValue::Int { value, min, max } = &mut item.value {
        *value = (*value + delta).clamp(*min, *max);
    }
}

fn flush_pending_events() {
    while event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) {
        let _ = event::read();
    }
}
