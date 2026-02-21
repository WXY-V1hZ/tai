use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};

pub struct ProviderEntry {
    pub name: String,
    pub base_url: String,
    pub api_key: String,
}

#[derive(Clone)]
enum Phase {
    ProviderList {
        selected: usize,
    },
    FieldList {
        provider_idx: usize,
        field_idx: usize,
    },
    FieldEdit {
        provider_idx: usize,
        field_idx: usize,
        input: String,
        input_row: u16,
    },
}

const FIELD_LABELS: [&str; 2] = ["API Key ", "Base URL"];

/// 交互式 Provider 配置编辑器
/// 返回 Some(updated) 若用户按 s 保存了修改，None 若取消退出
pub fn config_providers(mut providers: Vec<ProviderEntry>) -> io::Result<Option<Vec<ProviderEntry>>> {
    if providers.is_empty() {
        return Ok(None);
    }

    let mut stdout = io::stdout();
    let mut phase = Phase::ProviderList { selected: 0 };
    let mut dirty = false;

    terminal::enable_raw_mode()?;
    let _guard = RawModeGuard;

    let start_row = cursor::position()?.1;
    stdout.execute(cursor::Hide)?;

    render_provider_list(&mut stdout, &providers, 0, start_row)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match phase.clone() {
                Phase::ProviderList { selected } => match key.code {
                    KeyCode::Up => {
                        let s = if selected > 0 { selected - 1 } else { providers.len() - 1 };
                        phase = Phase::ProviderList { selected: s };
                        render_provider_list(&mut stdout, &providers, s, start_row)?;
                    }
                    KeyCode::Down => {
                        let s = if selected < providers.len() - 1 { selected + 1 } else { 0 };
                        phase = Phase::ProviderList { selected: s };
                        render_provider_list(&mut stdout, &providers, s, start_row)?;
                    }
                    KeyCode::Enter => {
                        phase = Phase::FieldList { provider_idx: selected, field_idx: 0 };
                        render_field_list(&mut stdout, &providers[selected], 0, start_row)?;
                    }
                    KeyCode::Char('s') => {
                        cleanup(&mut stdout, start_row)?;
                        stdout.execute(cursor::Show)?;
                        return Ok(if dirty { Some(providers) } else { None });
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        cleanup(&mut stdout, start_row)?;
                        stdout.execute(cursor::Show)?;
                        return Ok(None);
                    }
                    _ => {}
                },

                Phase::FieldList { provider_idx, field_idx } => match key.code {
                    KeyCode::Up => {
                        let f = if field_idx > 0 { field_idx - 1 } else { FIELD_LABELS.len() - 1 };
                        phase = Phase::FieldList { provider_idx, field_idx: f };
                        render_field_list(&mut stdout, &providers[provider_idx], f, start_row)?;
                    }
                    KeyCode::Down => {
                        let f = if field_idx < FIELD_LABELS.len() - 1 { field_idx + 1 } else { 0 };
                        phase = Phase::FieldList { provider_idx, field_idx: f };
                        render_field_list(&mut stdout, &providers[provider_idx], f, start_row)?;
                    }
                    KeyCode::Enter => {
                        let current_val = field_value(&providers[provider_idx], field_idx);
                        let input_row = render_field_edit_static(
                            &mut stdout,
                            &providers[provider_idx].name,
                            FIELD_LABELS[field_idx],
                            field_idx,
                            start_row,
                        )?;
                        render_field_edit_input(&mut stdout, &current_val, field_idx, input_row)?;
                        phase = Phase::FieldEdit { provider_idx, field_idx, input: current_val, input_row };
                    }
                    KeyCode::Char('s') => {
                        cleanup(&mut stdout, start_row)?;
                        stdout.execute(cursor::Show)?;
                        return Ok(if dirty { Some(providers) } else { None });
                    }
                    KeyCode::Esc => {
                        phase = Phase::ProviderList { selected: provider_idx };
                        render_provider_list(&mut stdout, &providers, provider_idx, start_row)?;
                    }
                    _ => {}
                },

                Phase::FieldEdit { provider_idx, field_idx, mut input, input_row } => {
                    match key.code {
                        KeyCode::Enter => {
                            let trimmed = input.trim().to_string();
                            if field_idx == 0 {
                                providers[provider_idx].api_key = trimmed;
                            } else if !trimmed.is_empty() {
                                providers[provider_idx].base_url = trimmed;
                            }
                            dirty = true;
                            phase = Phase::FieldList { provider_idx, field_idx };
                            render_field_list(&mut stdout, &providers[provider_idx], field_idx, start_row)?;
                        }
                        KeyCode::Esc => {
                            phase = Phase::FieldList { provider_idx, field_idx };
                            render_field_list(&mut stdout, &providers[provider_idx], field_idx, start_row)?;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            phase = Phase::FieldList { provider_idx, field_idx };
                            render_field_list(&mut stdout, &providers[provider_idx], field_idx, start_row)?;
                        }
                        KeyCode::Backspace => {
                            input.pop();
                            render_field_edit_input(&mut stdout, &input, field_idx, input_row)?;
                            phase = Phase::FieldEdit { provider_idx, field_idx, input, input_row };
                        }
                        KeyCode::Char(c) => {
                            input.push(c);
                            render_field_edit_input(&mut stdout, &input, field_idx, input_row)?;
                            phase = Phase::FieldEdit { provider_idx, field_idx, input, input_row };
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

// ── Rendering ────────────────────────────────────────────────────────────────

fn render_provider_list(
    stdout: &mut impl Write,
    providers: &[ProviderEntry],
    selected: usize,
    start_row: u16,
) -> io::Result<()> {
    stdout.queue(cursor::MoveTo(0, start_row))?;
    stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;

    stdout
        .queue(SetForegroundColor(Color::DarkGrey))?
        .queue(Print("  配置 Provider  (↑↓ 移动, Enter 选择, s 保存, Esc 退出)\n\n"))?
        .queue(ResetColor)?;

    for (i, p) in providers.iter().enumerate() {
        let is_sel = i == selected;
        let prefix = if is_sel { "> " } else { "  " };
        let (icon, status, status_color) = if p.api_key.is_empty() {
            ("✗", "API Key 未设置", Color::Yellow)
        } else {
            ("✓", "已配置", Color::Green)
        };

        if is_sel {
            stdout
                .queue(SetForegroundColor(Color::Cyan))?
                .queue(Print(format!("{}  {:<14}", prefix, p.name)))?
                .queue(ResetColor)?;
        } else {
            stdout.queue(Print(format!("{}  {:<14}", prefix, p.name)))?;
        }

        stdout
            .queue(SetForegroundColor(status_color))?
            .queue(Print(format!("{}  {}\n", icon, status)))?
            .queue(ResetColor)?;
    }

    stdout.flush()?;
    Ok(())
}

fn render_field_list(
    stdout: &mut impl Write,
    provider: &ProviderEntry,
    field_idx: usize,
    start_row: u16,
) -> io::Result<()> {
    stdout.queue(cursor::MoveTo(0, start_row))?;
    stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;

    stdout
        .queue(SetForegroundColor(Color::DarkGrey))?
        .queue(Print(format!(
            "  {}  (↑↓ 移动, Enter 编辑, s 保存, Esc 返回)\n\n",
            provider.name
        )))?
        .queue(ResetColor)?;

    let values: [String; 2] = [
        if provider.api_key.is_empty() {
            "未设置".to_string()
        } else {
            "●".repeat(provider.api_key.len().min(24))
        },
        provider.base_url.clone(),
    ];

    for (i, (label, value)) in FIELD_LABELS.iter().zip(values.iter()).enumerate() {
        let is_sel = i == field_idx;
        let prefix = if is_sel { "> " } else { "  " };
        let val_color = if i == 0 && provider.api_key.is_empty() {
            Color::Yellow
        } else {
            Color::DarkGrey
        };

        if is_sel {
            stdout
                .queue(SetForegroundColor(Color::Cyan))?
                .queue(Print(format!("{}  {}  ", prefix, label)))?
                .queue(Print(format!("{}\n", value)))?
                .queue(ResetColor)?;
        } else {
            stdout.queue(Print(format!("{}  {}  ", prefix, label)))?;
            stdout
                .queue(SetForegroundColor(val_color))?
                .queue(Print(format!("{}\n", value)))?
                .queue(ResetColor)?;
        }
    }

    stdout.flush()?;
    Ok(())
}

/// 绘制编辑界面静态部分，返回输入行所在行号
fn render_field_edit_static(
    stdout: &mut impl Write,
    provider_name: &str,
    field_label: &str,
    field_idx: usize,
    start_row: u16,
) -> io::Result<u16> {
    stdout.queue(cursor::MoveTo(0, start_row))?;
    stdout.queue(terminal::Clear(ClearType::FromCursorDown))?;

    // 标题
    stdout
        .queue(SetForegroundColor(Color::Yellow))?
        .queue(Print(format!("  {} › {}\n", provider_name, field_label.trim())))?
        .queue(ResetColor)?;

    let mut row_offset: u16 = 1;

    // API Key 字段额外显示获取链接
    if field_idx == 0 {
        if let Some(url) = provider_key_url(provider_name) {
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print(format!("  获取地址: {}\n", url)))?
                .queue(ResetColor)?;
            row_offset += 1;
        }
    }

    stdout.queue(Print("\n"))?;
    row_offset += 1;

    let input_row = start_row + row_offset;

    // 占位行（由 render_field_edit_input 填充）+ 空行 + 提示
    stdout.queue(Print("\n\n"))?;
    stdout
        .queue(SetForegroundColor(Color::DarkGrey))?
        .queue(Print("  Enter 确认  Esc 取消\n"))?
        .queue(ResetColor)?;

    stdout.flush()?;
    Ok(input_row)
}

/// 只更新输入行，不重绘其他内容
fn render_field_edit_input(
    stdout: &mut impl Write,
    input: &str,
    field_idx: usize,
    input_row: u16,
) -> io::Result<()> {
    stdout.queue(cursor::MoveTo(0, input_row))?;
    stdout.queue(terminal::Clear(ClearType::CurrentLine))?;

    if input.is_empty() {
        stdout
            .queue(SetForegroundColor(Color::DarkGrey))?
            .queue(Print("  > 粘贴或输入..."))?
            .queue(ResetColor)?;
    } else if field_idx == 0 {
        let masked: String = "●".repeat(input.len());
        stdout
            .queue(SetForegroundColor(Color::Cyan))?
            .queue(Print(format!("  > {}", masked)))?
            .queue(ResetColor)?;
    } else {
        stdout
            .queue(SetForegroundColor(Color::Cyan))?
            .queue(Print(format!("  > {}", input)))?
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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn field_value(provider: &ProviderEntry, field_idx: usize) -> String {
    match field_idx {
        0 => provider.api_key.clone(),
        _ => provider.base_url.clone(),
    }
}

fn provider_key_url(provider_name: &str) -> Option<&'static str> {
    match provider_name.to_lowercase().as_str() {
        "openai" => Some("https://platform.openai.com/api-keys"),
        "deepseek" => Some("https://platform.deepseek.com"),
        "anthropic" => Some("https://console.anthropic.com/settings/keys"),
        _ => None,
    }
}

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}
