use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::{
    command_parser::{CommandParser, Commands},
    commands::IntoCommand,
    components::GraphExplorer,
    state::SharedState,
    Msg,
};

use self::{
    command_bar::{CommandBar, CommandBarState},
    dialog::{CreateItem, CreateItemState},
};

mod command_bar;
pub mod dialog;

pub enum Dialog {
    CreateItem { state: CreateItemState },
}

impl Dialog {
    pub fn get_command(&self) -> Option<hyperlog_core::commander::Command> {
        match self {
            Dialog::CreateItem { state } => state.get_command(),
        }
    }
}

pub enum Mode {
    View,
    Insert,
    Command,
}

pub enum AppFocus {
    Dialog,
    Graph,
}

pub struct App<'a> {
    root: String,

    state: SharedState,

    pub mode: Mode,
    dialog: Option<Dialog>,
    command: Option<CommandBarState>,

    graph_explorer: GraphExplorer<'a>,

    focus: AppFocus,
}

impl<'a> App<'a> {
    pub fn new(
        root: impl Into<String>,
        state: SharedState,
        graph_explorer: GraphExplorer<'a>,
    ) -> Self {
        Self {
            root: root.into(),
            mode: Mode::View,
            dialog: None,
            command: None,
            state,
            graph_explorer,
            focus: AppFocus::Graph,
        }
    }

    pub fn update(&mut self, msg: Msg) -> anyhow::Result<impl IntoCommand> {
        tracing::trace!("handling msg: {:?}", msg);

        match msg {
            Msg::MoveRight => self.graph_explorer.move_right()?,
            Msg::MoveLeft => self.graph_explorer.move_left()?,
            Msg::MoveDown => self.graph_explorer.move_down()?,
            Msg::MoveUp => self.graph_explorer.move_up()?,
            Msg::OpenCreateItemDialog => self.open_dialog(),
            Msg::EnterInsertMode => self.mode = Mode::Insert,
            Msg::EnterViewMode => self.mode = Mode::View,
            Msg::EnterCommandMode => {
                self.command = Some(CommandBarState::default());
                self.mode = Mode::Command
            }
            Msg::Interact => match self.focus {
                AppFocus::Dialog => {}
                AppFocus::Graph => self.graph_explorer.interact()?,
            },
            Msg::SubmitCommand { command } => {
                tracing::info!("submitting command");

                if let Some(command) = CommandParser::parse(&command) {
                    match self.focus {
                        AppFocus::Dialog => {
                            if command.is_write() {
                                if let Some(dialog) = &self.dialog {
                                    if let Some(output) = dialog.get_command() {
                                        self.state.commander.execute(output)?;
                                    }
                                }

                                self.graph_explorer.update_graph()?;
                            }

                            if command.is_quit() {
                                self.dialog = None;
                            }
                        }
                        AppFocus::Graph => self.graph_explorer.execute_command(&command)?,
                    }
                }
                self.command = None;
                return Ok(Msg::EnterViewMode.into_command());
            }
            _ => {}
        }

        if let Some(command) = &mut self.command {
            let cmd = command.update(&msg)?;
            return Ok(cmd.into_command());
        } else if let Some(dialog) = &mut self.dialog {
            match dialog {
                Dialog::CreateItem { state } => state.update(&msg)?,
            }
        }

        Ok(().into_command())
    }

    fn open_dialog(&mut self) {
        if self.dialog.is_none() {
            let root = self.root.clone();
            let path = self.graph_explorer.get_current_path();

            self.dialog = Some(Dialog::CreateItem {
                state: CreateItemState::new(root, path),
            });
        }
    }
}

impl<'a> Widget for &mut App<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        StatefulWidget::render(
            GraphExplorer::new(self.state.clone()),
            area,
            buf,
            &mut self.graph_explorer.inner,
        )
    }
}

pub fn render_app(frame: &mut Frame, state: &mut App) {
    let chunks = Layout::vertical(vec![
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(frame.size());

    let mut heading_parts = vec![Span::styled("hyperlog", Style::default()).fg(Color::Green)];

    if let Some(dialog) = &state.dialog {
        heading_parts.push(Span::raw(" ~ "));

        match dialog {
            Dialog::CreateItem { .. } => heading_parts.push(Span::raw("create item")),
        }
    }

    let heading = Paragraph::new(text::Line::from(heading_parts));
    let block_heading = Block::default().borders(Borders::BOTTOM);

    frame.render_widget(heading.block(block_heading), chunks[0]);

    match &state.mode {
        Mode::View => {
            let line = Line::raw("-- VIEW --");

            let powerbar_block = Block::default()
                .borders(Borders::empty())
                .padding(Padding::new(1, 1, 0, 0));
            frame.render_widget(Paragraph::new(vec![line]).block(powerbar_block), chunks[2]);
        }
        Mode::Insert => {
            let line = Line::raw("-- EDIT --");

            let powerbar_block = Block::default()
                .borders(Borders::empty())
                .padding(Padding::new(1, 1, 0, 0));
            frame.render_widget(Paragraph::new(vec![line]).block(powerbar_block), chunks[2]);
        }
        Mode::Command => {
            if let Some(command) = &mut state.command {
                frame.render_stateful_widget(CommandBar::default(), chunks[2], command);
            }
        }
    }

    let Rect { width, height, .. } = chunks[1];

    let height = height as usize;
    let width = width as usize;

    let mut lines = Vec::new();
    for y in 0..height {
        if !y % 2 == 0 {
            lines.push(text::Line::default());
        } else {
            lines.push(text::Line::raw(" ~ ".repeat(width / 3)));
        }
    }
    let _background = Paragraph::new(lines);

    let _bg_block = Block::default()
        .fg(Color::DarkGray)
        .bold()
        .padding(Padding {
            left: 4,
            right: 4,
            top: 2,
            bottom: 2,
        });

    if let Some(dialog) = state.dialog.as_mut() {
        match dialog {
            Dialog::CreateItem { state } => {
                frame.render_stateful_widget(&mut CreateItem::default(), chunks[1], state)
            }
        }

        return;
    }

    frame.render_widget(state, chunks[1]);
}
