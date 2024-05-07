use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::{components::GraphExplorer, models::EditMsg, state::SharedState, Msg};

use self::dialog::{CreateItem, CreateItemState};

pub mod dialog;

pub enum Dialog {
    CreateItem { state: CreateItemState },
}

pub enum Mode {
    View,
    Insert,
}

pub struct App<'a> {
    state: SharedState,

    pub mode: Mode,
    dialog: Option<Dialog>,

    graph_explorer: GraphExplorer<'a>,
}

impl<'a> App<'a> {
    pub fn new(state: SharedState, graph_explorer: GraphExplorer<'a>) -> Self {
        Self {
            mode: Mode::View,
            dialog: None,
            state,
            graph_explorer,
        }
    }

    pub fn update(&mut self, msg: Msg) -> anyhow::Result<()> {
        tracing::trace!("handling msg: {:?}", msg);

        match msg {
            Msg::MoveRight => self.graph_explorer.move_right()?,
            Msg::MoveLeft => self.graph_explorer.move_left()?,
            Msg::MoveDown => self.graph_explorer.move_down()?,
            Msg::MoveUp => self.graph_explorer.move_up()?,
            Msg::OpenCreateItemDialog => self.open_dialog(),
            Msg::EnterInsertMode => self.mode = Mode::Insert,
            Msg::EnterCommandMode => self.mode = Mode::View,
            _ => {}
        }

        if let Some(dialog) = &mut self.dialog {
            match dialog {
                Dialog::CreateItem { state } => state.update(&msg)?,
            }
        }

        Ok(())
    }

    fn open_dialog(&mut self) {
        if self.dialog.is_none() {
            self.dialog = Some(Dialog::CreateItem {
                state: CreateItemState::default(),
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

    let powerbar = match &state.mode {
        Mode::View => Line::raw("-- VIEW --"),
        Mode::Insert => Line::raw("-- EDIT --"),
    };
    let powerbar_block = Block::default()
        .borders(Borders::empty())
        .padding(Padding::new(1, 1, 0, 0));
    frame.render_widget(
        Paragraph::new(vec![powerbar]).block(powerbar_block),
        chunks[2],
    );

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
