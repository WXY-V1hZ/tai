use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};

pub struct ModelItem {
    pub provider: String,
    pub model: String,
}

impl ModelItem {
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
        }
    }

    pub fn display(&self) -> String {
        format!("{}/{}", self.provider, self.model)
    }
}

/// 交互式模型选择器，支持上下方向键导航，回车确认
/// 返回选中的 ModelItem 索引，Esc/q 取消返回 None
pub fn select_model(items: &[ModelItem], current_index: usize) -> io::Result<Option<usize>> {
    if items.is_empty() {
        return Ok(None);
    }

    let mut stdout = io::stdout();
    let mut selected = current_index.min(items.len() - 1);

    terminal::enable_raw_mode()?;
    let _guard = RawModeGuard;

    // 记录起始行
    let start_row = cursor::position()?.1;

    // 渲染提示
    stdout.execute(cursor::Hide)?;

    render_list(&mut stdout, items, selected, start_row, current_index)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                    } else {
                        selected = items.len() - 1;
                    }
                    render_list(&mut stdout, items, selected, start_row, current_index)?;
                }
                KeyCode::Down => {
                    if selected < items.len() - 1 {
                        selected += 1;
                    } else {
                        selected = 0;
                    }
                    render_list(&mut stdout, items, selected, start_row, current_index)?;
                }
                KeyCode::Enter => {
                    cleanup(&mut stdout, items, start_row)?;
                    stdout.execute(cursor::Show)?;
                    return Ok(Some(selected));
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    cleanup(&mut stdout, items, start_row)?;
                    stdout.execute(cursor::Show)?;
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}

fn render_list(
    stdout: &mut impl Write,
    items: &[ModelItem],
    selected: usize,
    start_row: u16,
    current_index: usize,
) -> io::Result<()> {
    stdout.queue(cursor::MoveTo(0, start_row))?;
    stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;

    stdout
        .queue(SetForegroundColor(Color::DarkGrey))?
        .queue(Print("选择模型 (↑↓ 移动, Enter 确认, Esc 取消)\n"))?
        .queue(ResetColor)?;

    for (i, item) in items.iter().enumerate() {
        let is_selected = i == selected;
        let is_current = i == current_index;

        let prefix = if is_selected { "> " } else { "  " };
        let suffix = if is_current { " *" } else { "" };

        if is_selected {
            stdout
                .queue(SetForegroundColor(Color::Cyan))?
                .queue(Print(format!("{}{}{}\n", prefix, item.display(), suffix)))?
                .queue(ResetColor)?;
        } else if is_current {
            stdout
                .queue(SetForegroundColor(Color::Green))?
                .queue(Print(format!("{}{}{}\n", prefix, item.display(), suffix)))?
                .queue(ResetColor)?;
        } else {
            stdout.queue(Print(format!("{}{}{}\n", prefix, item.display(), suffix)))?;
        }
    }

    stdout.flush()?;
    Ok(())
}

fn cleanup(stdout: &mut impl Write, items: &[ModelItem], start_row: u16) -> io::Result<()> {
    stdout.queue(cursor::MoveTo(0, start_row))?;
    stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;
    let _ = items;
    stdout.flush()?;
    Ok(())
}

/// RAII 守卫：确保在退出时恢复终端模式
struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}
