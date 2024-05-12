use hyperlog_core::log::GraphItem;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use crate::{
    commands::{update_item::UpdateItemCommandExt, IntoCommand},
    models::Msg,
    state::SharedState,
};

use super::{InputBuffer, InputField};

enum EditItemFocused {
    Title,
    Description,
}
impl Default for EditItemFocused {
    fn default() -> Self {
        Self::Title
    }
}

pub struct EditItemState {
    root: String,
    path: Vec<String>,

    title: InputBuffer,
    description: InputBuffer,

    item: GraphItem,

    focused: EditItemFocused,

    state: SharedState,
}

impl EditItemState {
    pub fn new(
        state: &SharedState,
        root: impl Into<String>,
        path: impl IntoIterator<Item = impl Into<String>>,
        item: &GraphItem,
    ) -> Self {
        let root = root.into();
        let path = path.into_iter().map(|p| p.into()).collect_vec();

        match item {
            GraphItem::Item {
                title, description, ..
            } => {
                let title_len = title.len();
                let mut title = InputBuffer::new(title.clone());
                title.transform_focused();
                title.set_position(title_len);

                Self {
                    state: state.clone(),

                    root,
                    path,

                    item: item.clone(),

                    title,
                    description: InputBuffer::new(description.clone()),
                    focused: Default::default(),
                }
            }
            _ => todo!("cannot edit item from other than GraphItem::Item"),
        }
    }

    pub fn update(&mut self, msg: &Msg) -> anyhow::Result<()> {
        match &msg {
            Msg::MoveDown | Msg::MoveUp => match self.focused {
                EditItemFocused::Title => self.focused = EditItemFocused::Description,
                EditItemFocused::Description => self.focused = EditItemFocused::Title,
            },
            _ => {}
        }

        match self.focused {
            EditItemFocused::Title => {
                self.title.update(msg)?;
            }
            EditItemFocused::Description => {
                self.description.update(msg)?;
            }
        }

        Ok(())
    }

    pub fn get_command(&self) -> Option<impl IntoCommand> {
        let title = self.title.string();
        let description = self.description.string();

        if !title.is_empty() {
            let path = self.path.clone();

            Some(self.state.update_item_command().command(
                &self.root,
                &path.iter().map(|s| s.as_str()).collect_vec(),
                title.trim(),
                description.trim(),
                match &self.item {
                    GraphItem::User(_) => Default::default(),
                    GraphItem::Section(_) => Default::default(),
                    GraphItem::Item { state, .. } => state.clone(),
                },
            ))

            // Some(commander::Command::UpdateItem {
            //     root: self.root.clone(),
            //     path,
            //     title: title.trim().into(),
            //     description: description.trim().into(),
            //     state: match &self.item {
            //         GraphItem::User(_) => Default::default(),
            //         GraphItem::Section(_) => Default::default(),
            //         GraphItem::Item { state, .. } => state.clone(),
            //     },
            // })
        } else {
            None
        }
    }
}

#[derive(Default)]
pub struct EditItem {}

impl StatefulWidget for &mut EditItem {
    type State = EditItemState;

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
            EditItemFocused::Title => title_input.focused = true,
            EditItemFocused::Description => description_input.focused = true,
        }

        title_input.render(chunks[1], buf, &mut state.title);
        description_input.render(chunks[2], buf, &mut state.description);
    }
}
