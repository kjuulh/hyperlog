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

pub struct InputBuffer {
    pub state: BufferState,
}

impl InputBuffer {
    fn to_focused(&mut self) {
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

    fn to_static(&mut self) {
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
            Msg::EnterInsertMode => self.to_focused(),
            Msg::EnterCommandMode => self.to_focused(),
            Msg::EnterViewMode => self.to_static(),
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
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self {
            state: BufferState::default(),
        }
    }
}

pub struct InputField<'a> {
    title: &'a str,
}

impl<'a> InputField<'a> {
    pub fn new(title: &'a str) -> Self {
        Self { title }
    }
}

impl<'a> StatefulWidget for InputField<'a> {
    type State = InputBuffer;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered().title(self.title);

        match &state.state {
            BufferState::Focused { content, position } => {
                Paragraph::new(content.to_string().as_str())
                    .block(block)
                    .render(area, buf);

                buf.get_mut(area.x + 1 + *position as u16, area.y + 1)
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

enum CreateItemFocused {
    Title,
    Description,
}
impl Default for CreateItemFocused {
    fn default() -> Self {
        Self::Title
    }
}

#[derive(Default)]
pub struct CreateItemState {
    title: InputBuffer,
    description: InputBuffer,

    focused: CreateItemFocused,
}

impl CreateItemState {
    pub fn update(&mut self, msg: &Msg) -> anyhow::Result<()> {
        match &msg {
            Msg::MoveDown | Msg::MoveUp => match self.focused {
                CreateItemFocused::Title => self.focused = CreateItemFocused::Description,
                CreateItemFocused::Description => self.focused = CreateItemFocused::Title,
            },
            _ => {}
        }

        match self.focused {
            CreateItemFocused::Title => {
                self.title.update(msg)?;
            }
            CreateItemFocused::Description => {
                self.description.update(msg)?;
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct CreateItem {}

impl StatefulWidget for &mut CreateItem {
    // fn render(self, area: Rect, buf: &mut Buffer)
    // where
    //     Self: Sized,
    // {
    //     //buf.reset();

    //     // let block = Block::bordered()
    //     //     .title("create item")
    //     //     .padding(Padding::proportional(1));

    // }

    type State = CreateItemState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks =
            Layout::vertical(vec![Constraint::Length(3), Constraint::Length(3)]).split(area);

        InputField::new("title").render(chunks[0], buf, &mut state.title);
        InputField::new("description").render(chunks[1], buf, &mut state.description);

        // let title = Paragraph::new("something"); //.block(block);

        // title.render(area, buf);
    }
}
