use ratatui::widgets::{Paragraph, StatefulWidget, Widget};

use crate::{
    commands::IntoCommand,
    models::{EditMsg, Msg},
};

use super::dialog::BufferState;

pub struct CommandBarState {
    contents: BufferState,
}

impl Default for CommandBarState {
    fn default() -> Self {
        Self {
            contents: BufferState::Focused {
                content: ropey::Rope::default(),
                position: 0,
            },
        }
    }
}

impl CommandBarState {
    pub fn update(&mut self, msg: &Msg) -> anyhow::Result<impl IntoCommand> {
        if let Msg::Edit(e) = msg {
            self.contents.update(e)?;

            if let EditMsg::InsertNewLine = e {
                return Ok(Msg::SubmitCommand {
                    command: self.contents.string(),
                }
                .into_command());
            }
        }

        Ok(().into_command())
    }
}

#[derive(Default)]
pub struct CommandBar {}

impl StatefulWidget for CommandBar {
    type State = CommandBarState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        Paragraph::new(format!(":{}", state.contents.string())).render(area, buf);
    }
}
