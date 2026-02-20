use std::io::{self, Write};

use termimad::{
    crossterm::{
        style::{Color, Print, ResetColor, SetForegroundColor},
        QueueableCommand,
    },
    MadSkin,
};

/// 流式渲染器：reasoning 增量输出灰色文本，answer 收集后由 termimad 一次性渲染 Markdown
pub struct ReasoningDisplay {
    reasoning_buffer: String,
    answer_buffer: String,
    /// reasoning 已输出的字节数（用于增量追加）
    reasoning_rendered_bytes: usize,
    skin: MadSkin,
}

impl ReasoningDisplay {
    pub fn new() -> Self {
        let mut skin = MadSkin::default();
        skin.set_headers_fg(Color::Cyan);
        skin.bold.set_fg(Color::Yellow);
        skin.italic.set_fg(Color::Magenta);
        skin.code_block.set_fgbg(Color::White, Color::DarkGrey);
        skin.inline_code.set_fg(Color::Green);
        skin.table.set_fg(Color::Cyan);

        Self {
            reasoning_buffer: String::new(),
            answer_buffer: String::new(),
            reasoning_rendered_bytes: 0,
            skin,
        }
    }

    pub fn append_reasoning(&mut self, text: &str) {
        self.reasoning_buffer.push_str(text);
    }

    pub fn append_answer(&mut self, text: &str) {
        self.answer_buffer.push_str(text);
    }

    /// 流式阶段渲染入口：每次有新数据时调用
    /// - reasoning：增量输出灰色原文
    /// - answer：暂不输出，等 finish() 统一渲染
    pub fn render(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        self.render_reasoning_incremental(&mut stdout)?;
        stdout.flush()
    }

    /// 增量输出 reasoning 新增文本（灰色）
    fn render_reasoning_incremental(&mut self, stdout: &mut impl Write) -> io::Result<()> {
        let new_bytes = &self.reasoning_buffer.as_bytes()[self.reasoning_rendered_bytes..];
        if new_bytes.is_empty() {
            return Ok(());
        }
        // Safety: reasoning_buffer 是合法 UTF-8，切片按字节偏移量，追加时保证对齐
        let new_text = std::str::from_utf8(new_bytes).unwrap_or_default();

        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print(new_text))?;
        stdout.queue(ResetColor)?;

        self.reasoning_rendered_bytes = self.reasoning_buffer.len();
        Ok(())
    }

    /// 流式结束后调用：确保 reasoning 末尾换行，再用 termimad 渲染完整 Markdown answer
    pub fn finish(self) -> io::Result<()> {
        let mut stdout = io::stdout();

        if !self.reasoning_buffer.is_empty() && !self.reasoning_buffer.ends_with('\n') {
            writeln!(stdout)?;
        }

        if !self.answer_buffer.is_empty() {
            // reasoning 和 answer 之间加一个空行分隔
            if !self.reasoning_buffer.is_empty() {
                writeln!(stdout)?;
            }
            stdout.flush()?;
            self.skin.print_text(&self.answer_buffer);
        }

        writeln!(stdout)?;
        stdout.flush()
    }
}

impl Default for ReasoningDisplay {
    fn default() -> Self {
        Self::new()
    }
}
