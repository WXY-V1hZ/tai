use std::io::{self, Write};

use termimad::{
    crossterm::{
        style::{Color, Print, ResetColor, SetForegroundColor},
        QueueableCommand,
    },
};

use crate::viewer::{show_markdown_view, make_default_skin};

fn make_answer_skin() -> termimad::MadSkin {
    make_default_skin()
}

/// 流式渲染器
/// - 流式阶段：reasoning 灰色增量输出，answer 直接打印 raw markdown
/// - finish 后：进入 alternate screen，展示可滚动的 Markdown 渲染视图
pub struct TextRenderer {
    reasoning_buffer: String,
    answer_buffer: String,
    reasoning_rendered_bytes: usize,
    answer_rendered_bytes: usize,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            reasoning_buffer: String::new(),
            answer_buffer: String::new(),
            reasoning_rendered_bytes: 0,
            answer_rendered_bytes: 0,
        }
    }

    pub fn append_reasoning(&mut self, text: &str) {
        self.reasoning_buffer.push_str(text);
    }

    pub fn append_answer(&mut self, text: &str) {
        self.answer_buffer.push_str(text);
    }

    /// 流式阶段调用：reasoning 灰色增量输出，answer 直接打印原始文本
    pub fn render(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();
        self.flush_reasoning(&mut stdout)?;
        self.flush_answer_raw(&mut stdout)?;
        stdout.flush()
    }

    fn flush_reasoning(&mut self, stdout: &mut impl Write) -> io::Result<()> {
        let new_bytes = &self.reasoning_buffer.as_bytes()[self.reasoning_rendered_bytes..];
        if new_bytes.is_empty() {
            return Ok(());
        }
        let new_text = std::str::from_utf8(new_bytes).unwrap_or_default();
        stdout.queue(SetForegroundColor(Color::DarkGrey))?;
        stdout.queue(Print(new_text))?;
        stdout.queue(ResetColor)?;
        self.reasoning_rendered_bytes = self.reasoning_buffer.len();
        Ok(())
    }

    fn flush_answer_raw(&mut self, stdout: &mut impl Write) -> io::Result<()> {
        let new_bytes = &self.answer_buffer.as_bytes()[self.answer_rendered_bytes..];
        if new_bytes.is_empty() {
            return Ok(());
        }
        // 首次输出 answer 时，确保与 reasoning 之间有空行分隔
        if self.answer_rendered_bytes == 0 && !self.reasoning_buffer.is_empty() {
            let separator = if self.reasoning_buffer.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            stdout.queue(Print(separator))?;
        }
        let new_text = std::str::from_utf8(new_bytes).unwrap_or_default();
        stdout.queue(Print(new_text))?;
        self.answer_rendered_bytes = self.answer_buffer.len();
        Ok(())
    }

    /// 流式结束后调用，只返回 answer 部分的 markdown（不包含 reasoning）
    /// render_markdown: 是否进入 alternate screen 展示可滚动的渲染视图
    pub fn finish(self, render_markdown: bool) -> io::Result<String> {
        if self.answer_buffer.is_empty() {
            return Ok(String::new());
        }

        let mut stdout = io::stdout();
        if !self.answer_buffer.ends_with('\n') {
            writeln!(stdout)?;
        }
        stdout.flush()?;

        if render_markdown {
            show_markdown_view(&self.answer_buffer, make_answer_skin())?;
        }

        Ok(self.answer_buffer.clone())
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

