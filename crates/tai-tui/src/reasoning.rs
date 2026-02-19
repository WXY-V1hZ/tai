use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self},
    QueueableCommand,
};
use std::io::{self, Write};
use termimad::{MadSkin, FmtText};

pub struct ReasoningDisplay {
    reasoning_buffer: String,
    answer_buffer: String,
    has_printed_reasoning_header: bool,
    has_printed_answer_header: bool,
    start_row: u16,
    has_started: bool,
    skin: MadSkin,
    last_rendered_answer_len: usize,
    last_rendered_reasoning_len: usize,
}

impl ReasoningDisplay {
    pub fn new() -> Self {
        let mut skin = MadSkin::default();
        
        // 自定义皮肤样式
        skin.bold.set_fg(termimad::crossterm::style::Color::Yellow);
        skin.italic.set_fg(termimad::crossterm::style::Color::Cyan);
        skin.inline_code.set_fg(termimad::crossterm::style::Color::Green);
        skin.code_block.set_fg(termimad::crossterm::style::Color::Rgb { r: 200, g: 200, b: 200 });
        skin.headers[0].set_fg(termimad::crossterm::style::Color::Magenta);
        skin.headers[1].set_fg(termimad::crossterm::style::Color::Blue);
        
        Self {
            reasoning_buffer: String::new(),
            answer_buffer: String::new(),
            has_printed_reasoning_header: false,
            has_printed_answer_header: false,
            start_row: 0,
            has_started: false,
            skin,
            last_rendered_answer_len: 0,
            last_rendered_reasoning_len: 0,
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

        // 渲染思考过程（纯文本，灰色，增量式）
        if self.reasoning_buffer.len() > self.last_rendered_reasoning_len {
            if !self.has_printed_reasoning_header {
                stdout
                    .queue(SetForegroundColor(Color::DarkGrey))?
                    .queue(Print("【思考过程】\n"))?;
                self.has_printed_reasoning_header = true;
            }
            
            // 只打印新增的思考内容
            let new_reasoning = &self.reasoning_buffer[self.last_rendered_reasoning_len..];
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print(new_reasoning))?
                .queue(ResetColor)?;
            
            self.last_rendered_reasoning_len = self.reasoning_buffer.len();
        }

        // 渲染回答部分（Markdown）
        if self.answer_buffer.len() > self.last_rendered_answer_len {
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

            // 使用增量式渲染策略
            self.render_markdown_incrementally(&mut stdout)?;
        }

        stdout.flush()?;
        Ok(())
    }

    fn render_markdown_incrementally(&mut self, stdout: &mut impl Write) -> io::Result<()> {
        // 获取终端宽度
        let width = terminal::size()?.0 as usize;
        
        // 检查是否有新内容
        if self.answer_buffer.len() <= self.last_rendered_answer_len {
            return Ok(());
        }

        // 找到安全的渲染分割点
        let render_up_to = self.find_safe_render_point();
        
        if render_up_to <= self.last_rendered_answer_len {
            return Ok(()); // 没有可以安全渲染的新内容
        }

        // 确保 render_up_to 不超出缓冲区长度
        let render_up_to = render_up_to.min(self.answer_buffer.len());
        
        if render_up_to <= self.last_rendered_answer_len {
            return Ok(());
        }
        
        // 额外的边界检查
        if render_up_to > self.answer_buffer.len() {
            tracing::error!(
                "render_up_to ({}) exceeds buffer length ({})",
                render_up_to,
                self.answer_buffer.len()
            );
            return Ok(());
        }

        // 渲染新增的完整部分
        let content_to_render = &self.answer_buffer[self.last_rendered_answer_len..render_up_to];

        // 使用 termimad 渲染
        let fmt_text = FmtText::from(&self.skin, content_to_render, Some(width));
        
        write!(stdout, "{}", fmt_text)?;
        self.last_rendered_answer_len = render_up_to;

        Ok(())
    }

    /// 找到安全的渲染分割点，避免在代码块、列表、表格等结构中间分割
    fn find_safe_render_point(&self) -> usize {
        let content = &self.answer_buffer;
        let search_start = self.last_rendered_answer_len;
        
        // 如果内容很短，等待更多内容
        if content.len() - search_start < 10 {
            return self.last_rendered_answer_len;
        }

        // 分析当前状态：是否在特殊块内
        let state = self.analyze_markdown_state(content);
        
        // 如果在代码块内，必须等到代码块结束
        if state.in_code_block {
            // 从搜索起点之后查找代码块结束标记
            if let Some(pos) = self.find_code_block_end(content, search_start) {
                return pos.min(content.len());
            }
            // 如果没找到结束标记，保持原样不渲染
            return self.last_rendered_answer_len;
        }
        
        // 如果在表格内，等到表格结束
        if state.in_table {
            if let Some(pos) = self.find_table_end(content, search_start) {
                return pos.min(content.len());
            }
            return self.last_rendered_answer_len;
        }
        
        // 特别检查：如果未渲染部分即将开始一个代码块，等待整个代码块完成
        // 这是为了避免将代码块的开始标记和代码内容分开渲染
        let unrendered = &content[search_start..];
        let lines: Vec<&str> = unrendered.lines().collect();
        if let Some(first_line) = lines.first() {
            if first_line.trim_start().starts_with("```") {
                // 找到代码块的结束标记
                if let Some(pos) = self.find_code_block_end(content, search_start) {
                    return pos.min(content.len());
                }
                // 如果没找到结束标记，等待更多内容
                return self.last_rendered_answer_len;
            }
        }

        // 不在特殊块内，按优先级寻找安全分割点
        
        // 1. 优先寻找段落分隔（双换行），但要确保不在代码块内
        if let Some(pos) = self.find_safe_paragraph_break(content, search_start) {
            return pos.min(content.len());
        }
        
        // 2. 寻找标题行（以 # 开头的行）
        if let Some(pos) = self.find_next_heading(content, search_start) {
            if pos > search_start + 20 {
                return pos.min(content.len());
            }
        }
        
        // 3. 寻找列表项的自然结束点（多个列表项之后的空行）
        if let Some(pos) = self.find_list_break(content, search_start) {
            if pos > search_start + 20 {
                return pos.min(content.len());
            }
        }
        
        // 4. 如果有足够内容，按单行分割（但要确保不在代码块内）
        let new_content = &content[search_start..];
        if let Some(pos) = new_content.rfind('\n') {
            let absolute_pos = search_start + pos + 1;
            if absolute_pos - self.last_rendered_answer_len > 30 {
                // 检查这个位置是否在代码块外
                let content_up_to = &content[..absolute_pos.min(content.len())];
                let fence_count = content_up_to.lines()
                    .filter(|line| line.trim_start().starts_with("```"))
                    .count();
                // 只有在代码块外才分割
                if fence_count % 2 == 0 {
                    return absolute_pos.min(content.len());
                }
            }
        }
        
        // 5. 如果内容很长且没有换行，在句子结束处分割
        if new_content.len() > 150 {
            for (i, ch) in new_content.char_indices().rev() {
                if matches!(ch, '。' | '.' | '！' | '!' | '？' | '?') {
                    let absolute_pos = search_start + i + ch.len_utf8();
                    if absolute_pos - self.last_rendered_answer_len > 30 {
                        // 检查这个位置是否在代码块外
                        let content_up_to = &content[..absolute_pos.min(content.len())];
                        let fence_count = content_up_to.lines()
                            .filter(|line| line.trim_start().starts_with("```"))
                            .count();
                        if fence_count % 2 == 0 {
                            return absolute_pos.min(content.len());
                        }
                    }
                }
            }
        }
        
        // 没有找到安全的分割点，保持原样
        self.last_rendered_answer_len
    }

    /// 分析从上次渲染位置到现在的 Markdown 状态
    fn analyze_markdown_state(&self, content: &str) -> MarkdownState {
        let mut state = MarkdownState {
            in_code_block: false,
            in_table: false,
        };
        
        // 只分析已渲染部分，检查是否有未闭合的代码块
        let rendered_part = &content[..self.last_rendered_answer_len.min(content.len())];
        
        // 统计已渲染部分的代码围栏数量（只统计行首的 ```）
        let fence_count = rendered_part.lines()
            .filter(|line| line.trim_start().starts_with("```"))
            .count();
        state.in_code_block = fence_count % 2 != 0;
        
        // 如果不在代码块内，检查未渲染部分是否刚开始一个代码块
        if !state.in_code_block && self.last_rendered_answer_len < content.len() {
            let unrendered_part = &content[self.last_rendered_answer_len..];
            let unrendered_lines: Vec<&str> = unrendered_part.lines().collect();
            
            // 检查未渲染部分是否以代码块开始标记开头
            if let Some(first_line) = unrendered_lines.first() {
                if first_line.trim_start().starts_with("```") {
                    // 检查是否有结束标记
                    let fence_in_unrendered = unrendered_lines.iter()
                        .filter(|line| line.trim_start().starts_with("```"))
                        .count();
                    // 如果未渲染部分只有一个 ```（开始标记），说明代码块未完成
                    if fence_in_unrendered == 1 {
                        state.in_code_block = true;
                    }
                }
            }
        }
        
        // 检查是否在表格内（只检查未渲染部分的最近几行）
        if self.last_rendered_answer_len < content.len() {
            let check_from = self.last_rendered_answer_len.saturating_sub(200);
            // 确保 check_from 在字符边界上
            let check_from = self.find_char_boundary(content, check_from);
            let recent_content = &content[check_from..];
            let lines: Vec<&str> = recent_content.lines().collect();
            
            // 从后往前查找，看最近的非空行是否是表格行
            for line in lines.iter().rev().take(5) {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    break; // 遇到空行，表格结束
                }
                if trimmed.starts_with('|') && trimmed.ends_with('|') {
                    state.in_table = true;
                    break;
                }
                if !trimmed.starts_with('|') {
                    break; // 非表格行
                }
            }
        }
        
        state
    }

    /// 查找代码块结束位置（包含结束的 ``` 行）
    fn find_code_block_end(&self, content: &str, from: usize) -> Option<usize> {
        let remaining = &content[from..];
        
        // 首先跳过当前行（可能是代码块开始标记）
        let first_line_end = remaining.find('\n').map(|pos| pos + 1).unwrap_or(0);
        if first_line_end == 0 {
            return None; // 没有找到换行符，说明只有一行
        }
        
        let search_from = first_line_end;
        let search_content = &remaining[search_from..];
        
        // 逐字符遍历，寻找下一个行首的 ```
        let mut line_start = true;
        
        for (i, ch) in search_content.char_indices() {
            if line_start && search_content[i..].starts_with("```") {
                // 找到代码块结束标记，找到这一行的末尾
                if let Some(newline_pos) = search_content[i..].find('\n') {
                    return Some(from + search_from + i + newline_pos + 1);
                } else {
                    // 如果是最后一行（没有换行符），返回字符串结束位置
                    return Some(from + remaining.len());
                }
            }
            
            if ch == '\n' {
                line_start = true;
            } else if !ch.is_whitespace() {
                line_start = false;
            }
        }
        None
    }

    /// 查找表格结束位置（表格后的空行或非表格行）
    fn find_table_end(&self, content: &str, from: usize) -> Option<usize> {
        let remaining = &content[from..];
        let lines: Vec<&str> = remaining.lines().collect();
        
        let mut pos = 0;
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // 表格行必须以 | 开头和结尾
            if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
                // 遇到非表格行
                if i > 0 {
                    // 确保不超出范围
                    return Some((from + pos).min(content.len()));
                }
                break;
            }
            
            // 注意：最后一行可能没有换行符
            pos += line.len();
            if from + pos < content.len() {
                pos += 1; // 加上换行符
            }
        }
        None
    }

    /// 查找安全的段落分隔（双换行），确保不在代码块内
    fn find_safe_paragraph_break(&self, content: &str, from: usize) -> Option<usize> {
        let remaining = &content[from..];
        
        // 查找所有的 \n\n 位置
        let mut search_pos = 0;
        while let Some(pos) = remaining[search_pos..].find("\n\n") {
            let absolute_pos = from + search_pos + pos + 2;
            
            // 检查这个位置是否在代码块内
            // 统计从开始到这个位置之间的代码围栏数量
            let content_up_to = &content[..absolute_pos];
            let fence_count = content_up_to.lines()
                .filter(|line| line.trim_start().starts_with("```"))
                .count();
            
            // 如果是偶数个围栏，说明不在代码块内
            if fence_count % 2 == 0 {
                return Some(absolute_pos);
            }
            
            // 继续搜索下一个 \n\n
            search_pos = search_pos + pos + 2;
        }
        
        None
    }

    /// 查找下一个标题行的位置
    fn find_next_heading(&self, content: &str, from: usize) -> Option<usize> {
        let remaining = &content[from..];
        let lines: Vec<&str> = remaining.lines().collect();
        
        let mut pos = 0;
        for (i, line) in lines.iter().enumerate().skip(1) { // 跳过当前行
            let prev_line_len = lines[i - 1].len();
            pos += prev_line_len;
            // 只有在不是最后一行时才加换行符
            if from + pos < content.len() {
                pos += 1;
            }
            
            if line.trim_start().starts_with('#') {
                return Some((from + pos).min(content.len()));
            }
        }
        None
    }

    /// 查找列表的自然断点
    fn find_list_break(&self, content: &str, from: usize) -> Option<usize> {
        let remaining = &content[from..];
        let lines: Vec<&str> = remaining.lines().collect();
        
        let mut in_list = false;
        let mut pos = 0;
        
        for line in lines.iter() {
            let trimmed = line.trim_start();
            let is_list_item = trimmed.starts_with("- ") 
                || trimmed.starts_with("* ")
                || trimmed.starts_with("+ ")
                || (trimmed.len() > 2 && trimmed.chars().next().unwrap().is_numeric() && trimmed.chars().nth(1) == Some('.'));
            
            if is_list_item {
                in_list = true;
            } else if in_list && trimmed.is_empty() {
                // 列表后的空行
                let result = from + pos + line.len();
                // 只有在不超出范围时才加换行符
                let result = if result < content.len() { result + 1 } else { result };
                return Some(result.min(content.len()));
            } else if in_list && !trimmed.is_empty() && !is_list_item {
                // 列表结束，遇到非列表内容
                return Some((from + pos).min(content.len()));
            }
            
            pos += line.len();
            // 只有在不是最后一行时才加换行符
            if from + pos < content.len() {
                pos += 1;
            }
        }
        None
    }

    /// 确保索引位置在 UTF-8 字符边界上
    /// 如果不在边界上，向前调整到最近的字符边界
    fn find_char_boundary(&self, s: &str, index: usize) -> usize {
        if index >= s.len() {
            return s.len();
        }
        
        let mut boundary = index;
        while boundary > 0 && !s.is_char_boundary(boundary) {
            boundary -= 1;
        }
        boundary
    }

    pub fn finish(self) -> io::Result<()> {
        let mut stdout = io::stdout();

        // 先渲染剩余未渲染的思考内容（如果有的话）
        if self.reasoning_buffer.len() > self.last_rendered_reasoning_len {
            let remaining_reasoning = &self.reasoning_buffer[self.last_rendered_reasoning_len..];
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print(remaining_reasoning))?
                .queue(ResetColor)?;
        }

        // 渲染剩余未渲染的回答内容（如果有的话）
        if self.answer_buffer.len() > self.last_rendered_answer_len {
            let remaining = &self.answer_buffer[self.last_rendered_answer_len..];
            let width = terminal::size()?.0 as usize;
            let fmt_text = FmtText::from(&self.skin, remaining, Some(width - 2));
            write!(stdout, "{}", fmt_text)?;
        }

        // 打印换行符，确保输出完整
        stdout.queue(Print("\n"))?;
        
        // 如果有思考过程，在最后提示已折叠
        if !self.reasoning_buffer.is_empty() {
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(ResetColor)?;
        }
        
        stdout.flush()?;
        Ok(())
    }
}

#[derive(Debug)]
struct MarkdownState {
    in_code_block: bool,
    in_table: bool,
}

impl Default for ReasoningDisplay {
    fn default() -> Self {
        Self::new()
    }
}
