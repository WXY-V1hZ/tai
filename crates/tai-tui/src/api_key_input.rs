use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};

fn provider_key_url(provider_name: &str) -> Option<&'static str> {
    match provider_name.to_lowercase().as_str() {
        "openai" => Some("https://platform.openai.com/api-keys"),
        "deepseek" => Some("https://platform.deepseek.com"),
        "anthropic" => Some("https://console.anthropic.com/settings/keys"),
        _ => None,
    }
}

/// 显示 API Key 输入界面，返回用户输入的值，Esc/Ctrl+C 取消返回 None
pub fn prompt_api_key(provider_name: &str) -> io::Result<Option<String>> {
    let mut stdout = io::stdout();
    let mut input = String::new();

    terminal::enable_raw_mode()?;
    let _guard = RawModeGuard;

    let start_row = cursor::position()?.1;
    stdout.execute(cursor::Hide)?;

    // 静态部分只绘制一次，返回输入行所在行号
    let input_row = draw_static(&mut stdout, provider_name, start_row)?;
    draw_input_line(&mut stdout, &input, input_row)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Enter => {
                    let value = input.trim().to_string();
                    cleanup(&mut stdout, start_row)?;
                    stdout.execute(cursor::Show)?;
                    return Ok(if value.is_empty() { None } else { Some(value) });
                }
                KeyCode::Esc => {
                    cleanup(&mut stdout, start_row)?;
                    stdout.execute(cursor::Show)?;
                    return Ok(None);
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    cleanup(&mut stdout, start_row)?;
                    stdout.execute(cursor::Show)?;
                    return Ok(None);
                }
                KeyCode::Backspace => {
                    input.pop();
                    draw_input_line(&mut stdout, &input, input_row)?;
                }
                KeyCode::Char(c) => {
                    input.push(c);
                    draw_input_line(&mut stdout, &input, input_row)?;
                }
                _ => {}
            }
        }
    }
}

/// 绘制静态部分（标题、链接、底部提示），返回输入行的行号
fn draw_static(
    stdout: &mut impl Write,
    provider_name: &str,
    start_row: u16,
) -> io::Result<u16> {
    stdout.queue(cursor::MoveTo(0, start_row))?;
    stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;

    // 标题
    stdout
        .queue(SetForegroundColor(Color::Yellow))?
        .queue(Print(format!("  {} 需要 API Key\n", provider_name)))?
        .queue(ResetColor)?;

    let mut row_offset: u16 = 1;

    // 获取地址提示（可选行）
    if let Some(url) = provider_key_url(provider_name) {
        stdout
            .queue(SetForegroundColor(Color::DarkGrey))?
            .queue(Print(format!("  获取地址: {}\n", url)))?
            .queue(ResetColor)?;
        row_offset += 1;
    }

    // 空行 + 输入行占位 + 空行
    stdout.queue(Print("\n"))?;
    row_offset += 1;
    let input_row = start_row + row_offset;

    stdout.queue(Print("\n"))?; // 输入行占位
    stdout.queue(Print("\n"))?; // 输入行后空行

    // 底部操作提示
    stdout
        .queue(SetForegroundColor(Color::DarkGrey))?
        .queue(Print("  Enter 确认  Esc 取消\n"))?
        .queue(ResetColor)?;

    stdout.flush()?;
    Ok(input_row)
}

/// 只更新输入行，不影响其他行
fn draw_input_line(stdout: &mut impl Write, input: &str, input_row: u16) -> io::Result<()> {
    stdout.queue(cursor::MoveTo(0, input_row))?;
    stdout.queue(terminal::Clear(ClearType::CurrentLine))?;

    if input.is_empty() {
        stdout
            .queue(SetForegroundColor(Color::DarkGrey))?
            .queue(Print("  > 粘贴或输入 API Key..."))?
            .queue(ResetColor)?;
    } else {
        let masked: String = "●".repeat(input.len());
        stdout
            .queue(SetForegroundColor(Color::Cyan))?
            .queue(Print(format!("  > {}", masked)))?
            .queue(ResetColor)?;
    }

    stdout.flush()?;
    Ok(())
}

fn cleanup(stdout: &mut impl Write, start_row: u16) -> io::Result<()> {
    stdout.queue(cursor::MoveTo(0, start_row))?;
    stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;
    stdout.flush()?;
    Ok(())
}

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}
