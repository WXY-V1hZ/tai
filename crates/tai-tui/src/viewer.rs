use std::io::{self, Write};
use termimad::{
    crossterm::{
        cursor::{Hide, Show},
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        queue,
        style::Color,
        terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Area, MadSkin, MadView,
};

/// 创建默认的 Markdown 渲染皮肤
pub fn make_default_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.set_headers_fg(Color::Cyan);
    skin.bold.set_fg(Color::Yellow);
    skin.italic.set_fg(Color::Magenta);
    skin.table.set_fg(Color::Cyan);
    skin
}

/// 获取视图区域
fn view_area() -> Area {
    let mut area = Area::full_screen();
    area.pad_for_max_width(120);
    area
}

/// 进入 alternate screen，展示可滚动的 MadView，按任意键退出
pub fn show_markdown_view(markdown: &str, skin: MadSkin) -> io::Result<()> {
    let mut w = io::stdout();

    // raw mode 必须在 EnterAlternateScreen 之前启用，否则 Windows 终端可能无法读取键盘事件
    terminal::enable_raw_mode()?;
    queue!(w, EnterAlternateScreen, Hide)?;
    w.flush()?;

    // 清空流式输出期间积累的残留事件（例如用户输入命令时按下的回车）
    flush_pending_events();

    let mut view = MadView::from(markdown.to_owned(), view_area(), skin);

    let result = run_view_loop(&mut w, &mut view);

    terminal::disable_raw_mode()?;
    queue!(w, Show, LeaveAlternateScreen)?;
    w.flush()?;

    result
}

/// 视图交互循环
fn run_view_loop(w: &mut impl Write, view: &mut MadView) -> io::Result<()> {
    loop {
        view.write_on(w).map_err(|e| io::Error::other(e.to_string()))?;
        w.flush()?;

        match event::read() {
            Ok(Event::Key(KeyEvent { code, modifiers, kind: KeyEventKind::Press, .. })) => {
                // Ctrl+C 强制退出
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                match code {
                    KeyCode::Char('k') => view.try_scroll_lines(-1),
                    KeyCode::Up => view.try_scroll_lines(-1),
                    KeyCode::Char('j') => view.try_scroll_lines(1),
                    KeyCode::Down => view.try_scroll_lines(1),
                    KeyCode::PageUp => view.try_scroll_pages(-1),
                    KeyCode::PageDown => view.try_scroll_pages(1),
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
            Ok(Event::Key(_)) => {}
            Ok(Event::Resize(..)) => {
                queue!(w, Clear(ClearType::All))?;
                view.resize(&view_area());
            }
            _ => {}
        }
    }
    Ok(())
}

/// 消费掉 stdin 中所有尚未处理的积压事件，避免进入交互循环时被立即触发退出
fn flush_pending_events() {
    while event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) {
        let _ = event::read();
    }
}
