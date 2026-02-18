use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    QueueableCommand,
};
use std::io::{self, Write};

pub struct ReasoningDisplay {
    reasoning_buffer: String,
    answer_buffer: String,
    reasoning_printed_len: usize,
    answer_printed_len: usize,
    has_printed_reasoning_header: bool,
    has_printed_answer_header: bool,
    start_row: u16,
    has_started: bool,
}

impl ReasoningDisplay {
    pub fn new() -> Self {
        Self {
            reasoning_buffer: String::new(),
            answer_buffer: String::new(),
            reasoning_printed_len: 0,
            answer_printed_len: 0,
            has_printed_reasoning_header: false,
            has_printed_answer_header: false,
            start_row: 0,
            has_started: false,
        }
    }

    pub fn append_reasoning(&mut self, text: &str) {
        self.reasoning_buffer.push_str(text);
    }

    pub fn append_answer(&mut self, text: &str) {
        self.answer_buffer.push_str(text);
    }

    pub fn render(&mut self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // 第一次渲染时记录位置
        if !self.has_started {
            self.start_row = cursor::position()?.1;
            self.has_started = true;
        }

        // 如果有新的思考内容
        if self.reasoning_buffer.len() > self.reasoning_printed_len {
            if !self.has_printed_reasoning_header {
                stdout
                    .queue(SetForegroundColor(Color::DarkGrey))?
                    .queue(Print("【思考过程】\n"))?;
                self.has_printed_reasoning_header = true;
            }

            let new_reasoning = &self.reasoning_buffer[self.reasoning_printed_len..];
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print(new_reasoning))?
                .queue(ResetColor)?;
            
            self.reasoning_printed_len = self.reasoning_buffer.len();
        }

        // 如果有新的答案内容
        if self.answer_buffer.len() > self.answer_printed_len {
            if !self.has_printed_answer_header {
                // 如果之前有思考过程，先换行分隔
                if self.has_printed_reasoning_header {
                    stdout.queue(Print("\n\n"))?;
                }
                stdout
                    .queue(SetForegroundColor(Color::White))?
                    .queue(Print("【回答】\n"))?
                    .queue(ResetColor)?;
                self.has_printed_answer_header = true;
            }

            let new_answer = &self.answer_buffer[self.answer_printed_len..];
            stdout.queue(Print(new_answer))?;
            
            self.answer_printed_len = self.answer_buffer.len();
        }

        stdout.flush()?;
        Ok(())
    }

    pub fn finish(self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // 移动到起始位置
        stdout.queue(cursor::MoveTo(0, self.start_row))?;

        // 清除从当前位置到屏幕底部的所有内容
        stdout.queue(Clear(ClearType::FromCursorDown))?;

        // 如果有思考过程，折叠显示
        if !self.reasoning_buffer.is_empty() {
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print("[思考过程已折叠]\n\n"))?
                .queue(ResetColor)?;
        }

        // 显示最终答案
        stdout
            .queue(Print(&self.answer_buffer))?
            .queue(Print("\n"))?;

        stdout.flush()?;
        Ok(())
    }
}

impl Default for ReasoningDisplay {
    fn default() -> Self {
        Self::new()
    }
}
