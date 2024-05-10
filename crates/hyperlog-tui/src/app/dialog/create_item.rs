use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use crate::models::Msg;

use super::{InputBuffer, InputField};

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
            path.push(title.replace([' ', '.'], "-"));

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
    }
}
