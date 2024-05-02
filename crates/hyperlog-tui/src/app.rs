use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::{components::GraphExplorer, state::SharedState, Msg};

pub struct App<'a> {
    state: SharedState,

    graph_explorer: GraphExplorer<'a>,
}

impl<'a> App<'a> {
    pub fn new(state: SharedState, graph_explorer: GraphExplorer<'a>) -> Self {
        Self {
            state,
            graph_explorer,
        }
    }

    pub fn update(&mut self, msg: Msg) -> anyhow::Result<()> {
        tracing::trace!("handling msg: {:?}", msg);

        match msg {
            Msg::MoveRight => self.graph_explorer.move_right(),
            Msg::MoveLeft => self.graph_explorer.move_left(),
            Msg::MoveDown => self.graph_explorer.move_down(),
            Msg::MoveUp => self.graph_explorer.move_up(),
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
    let chunks =
        Layout::vertical(vec![Constraint::Length(2), Constraint::Min(0)]).split(frame.size());

    let heading = Paragraph::new(text::Line::from(
        Span::styled("hyperlog", Style::default()).fg(Color::Green),
    ));
    let block_heading = Block::default().borders(Borders::BOTTOM);

    frame.render_widget(heading.block(block_heading), chunks[0]);

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
    //frame.render_widget(background.block(bg_block), chunks[1]);

    frame.render_widget(state, chunks[1])
}
