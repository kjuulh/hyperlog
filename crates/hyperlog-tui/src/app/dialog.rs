use ratatui::{prelude::*, widgets::*};

use crate::models::{EditMsg, Msg};

pub enum BufferState {
    Focused {
        content: ropey::Rope,
        position: usize,
    },
    Static {
        content: String,
        position: usize,
    },
}

impl Default for BufferState {
    fn default() -> Self {
        Self::Static {
            content: String::new(),
            position: 0,
        }
    }
}

impl BufferState {
    pub fn update(&mut self, msg: &EditMsg) -> anyhow::Result<()> {
        if let BufferState::Focused { content, position } = self {
            let pos = *position;

            match msg {
                EditMsg::Delete => {
                    if pos > 0 && pos <= content.len_chars() {
                        content.remove((pos - 1)..pos);
                        *position = position.saturating_sub(1);
                    }
                }
                EditMsg::DeleteNext => {
                    if pos > 0 && pos < content.len_chars() {
                        content.remove((pos)..pos + 1);
                    }
                }
                EditMsg::InsertNewLine => {}
                EditMsg::InsertTab => {}
                EditMsg::InsertChar(c) => {
                    content.try_insert_char(pos, *c)?;
                    *position = position.saturating_add(1);
                }
                EditMsg::MoveLeft => {
                    *position = position.saturating_sub(1);
                }
                EditMsg::MoveRight => {
                    if pos < content.len_chars() {
                        *position = pos.saturating_add(1);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn string(&self) -> String {
        match self {
            BufferState::Focused { content, .. } => content.to_string(),
            BufferState::Static { content, .. } => content.to_owned(),
        }
    }
}

#[derive(Default)]
pub struct InputBuffer {
    pub state: BufferState,
}

impl InputBuffer {
    pub fn new(input: String) -> Self {
        Self {
            state: BufferState::Static {
                content: input,
                position: 0,
            },
        }
    }

    pub fn transform_focused(&mut self) {
        match &mut self.state {
            BufferState::Focused { .. } => {}
            BufferState::Static { content, position } => {
                self.state = BufferState::Focused {
                    content: ropey::Rope::from(content.as_str()),
                    position: *position,
                }
            }
        }
    }

    fn transform_static(&mut self) {
        match &mut self.state {
            BufferState::Focused { content, position } => {
                self.state = BufferState::Static {
                    content: content.to_string(),
                    position: *position,
                }
            }
            BufferState::Static { .. } => {}
        }
    }

    #[allow(dead_code)]
    pub fn toggle(&mut self) {
        match &mut self.state {
            BufferState::Focused { content, position } => {
                self.state = BufferState::Static {
                    content: content.to_string(),
                    position: *position,
                }
            }
            BufferState::Static { content, position } => {
                self.state = BufferState::Focused {
                    content: ropey::Rope::from(content.as_str()),
                    position: *position,
                }
            }
        }
    }

    pub fn update(&mut self, msg: &Msg) -> anyhow::Result<()> {
        match msg {
            Msg::EnterInsertMode => self.transform_focused(),
            Msg::EnterCommandMode => self.transform_focused(),
            Msg::EnterViewMode => self.transform_static(),
            Msg::Edit(c) => {
                self.state.update(c)?;
            }
            _ => {}
        }

        Ok(())
    }

    pub fn string(&self) -> String {
        match &self.state {
            BufferState::Focused { ref content, .. } => content.to_string(),
            BufferState::Static { content, .. } => content.to_owned(),
        }
    }

    fn set_position(&mut self, title_len: usize) {
        match &mut self.state {
            BufferState::Focused { position, .. } | BufferState::Static { position, .. } => {
                *position = title_len
            }
        }
    }
}

pub struct InputField<'a> {
    title: &'a str,

    focused: bool,
}

impl<'a> InputField<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            focused: false,
        }
    }
}

impl<'a> StatefulWidget for InputField<'a> {
    type State = InputBuffer;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let mut block = Block::bordered().title(self.title);

        if !self.focused {
            block = block.dark_gray();
        }

        match &state.state {
            BufferState::Focused { content, position } => {
                Paragraph::new(content.to_string().as_str())
                    .block(block)
                    .render(area, buf);

                buf.get_mut(clamp_x(&area, area.x + 1 + *position as u16), area.y + 1)
                    .set_style(Style::new().bg(Color::Magenta).fg(Color::Black));
            }
            BufferState::Static { content, .. } => {
                Paragraph::new(content.as_str())
                    .block(block)
                    .render(area, buf);
            }
        }
    }
}

fn clamp_x(area: &Rect, x: u16) -> u16 {
    if x >= area.width {
        area.width - 1
    } else {
        x
    }
}

pub mod create_item;
pub mod edit_item;
