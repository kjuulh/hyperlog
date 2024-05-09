use itertools::Itertools;
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
    fn transform_focused(&mut self) {
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

enum CreateItemFocused {
    Title,
    Description,
}
impl Default for CreateItemFocused {
    fn default() -> Self {
        Self::Title
    }
}

pub struct CreateItemState {
    root: String,
    path: Vec<String>,

    title: InputBuffer,
    description: InputBuffer,

    focused: CreateItemFocused,
}

impl CreateItemState {
    pub fn new(root: impl Into<String>, path: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let root = root.into();
        let path = path.into_iter().map(|p| p.into()).collect_vec();

        Self {
            root,
            path,

            title: Default::default(),
            description: Default::default(),
            focused: Default::default(),
        }
    }

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

    pub fn get_command(&self) -> Option<hyperlog_core::commander::Command> {
        let title = self.title.string();
        let description = self.description.string();

        if !title.is_empty() {
            let mut path = self.path.clone();
            path.push(title.replace(' ', "").replace('.', "-"));

            Some(hyperlog_core::commander::Command::CreateItem {
                root: self.root.clone(),
                path,
                title: title.trim().into(),
                description: description.trim().into(),
                state: hyperlog_core::log::ItemState::NotDone,
            })
        } else {
            None
        }
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
        let chunks = Layout::vertical(vec![
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

        let path = format!("path: {}.{}", state.root, state.path.join("."));
        let path_header = Paragraph::new(path).dark_gray();
        path_header.render(chunks[0], buf);

        let mut title_input = InputField::new("title");
        let mut description_input = InputField::new("description");

        match state.focused {
            CreateItemFocused::Title => title_input.focused = true,
            CreateItemFocused::Description => description_input.focused = true,
        }

        title_input.render(chunks[1], buf, &mut state.title);
        description_input.render(chunks[2], buf, &mut state.description);

        // let title = Paragraph::new("something"); //.block(block);

        // title.render(area, buf);
    }
}
