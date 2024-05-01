use std::{
    io::{self, Stdout},
    ops::{Deref, DerefMut},
    time::Duration,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hyperlog_core::{log::GraphItem, state::State};
use ratatui::{backend::CrosstermBackend, prelude::*, widgets::*, Frame, Terminal};

use crate::state::SharedState;

struct TerminalInstance {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalInstance {
    fn new() -> Result<Self> {
        Ok(Self {
            terminal: setup_terminal().context("setup failed")?,
        })
    }
}

impl Deref for TerminalInstance {
    type Target = Terminal<CrosstermBackend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for TerminalInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for TerminalInstance {
    fn drop(&mut self) {
        if let Err(e) = restore_terminal(&mut self.terminal).context("restore terminal failed") {
            tracing::error!("failed to restore terminal: {}", e);
        }
    }
}

mod state {
    use std::{ops::Deref, sync::Arc};

    use hyperlog_core::state::State;

    #[derive(Clone)]
    pub struct SharedState {
        state: Arc<State>,
    }

    impl Deref for SharedState {
        type Target = State;

        fn deref(&self) -> &Self::Target {
            &self.state
        }
    }

    impl From<State> for SharedState {
        fn from(value: State) -> Self {
            Self {
                state: Arc::new(value),
            }
        }
    }
}

pub async fn execute(state: State) -> Result<()> {
    tracing::debug!("starting hyperlog tui");

    logging::initialize_panic_handler()?;
    logging::initialize_logging()?;

    let state = SharedState::from(state);

    let mut terminal = TerminalInstance::new()?;
    run(&mut terminal, state).context("app loop failed")?;

    Ok(())
}

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

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, state: SharedState) -> Result<()> {
    let mut graph_explorer = GraphExplorer::new(state.clone());
    graph_explorer.update_graph()?;

    let mut app = App::new(state.clone(), graph_explorer);

    loop {
        terminal.draw(|f| crate::render_app(f, &mut app))?;
        if should_quit()? {
            break;
        }
    }
    Ok(())
}

fn render_app(frame: &mut Frame, state: &mut App) {
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
    let background = Paragraph::new(lines);

    let bg_block = Block::default()
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

struct GraphExplorer<'a> {
    state: SharedState,

    inner: GraphExplorerState<'a>,
}

struct GraphExplorerState<'a> {
    current_path: Option<&'a str>,

    graph: Option<GraphItem>,
}

impl GraphExplorer<'_> {
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            inner: GraphExplorerState::<'_> {
                current_path: None,
                graph: None,
            },
        }
    }

    pub fn update_graph(&mut self) -> Result<&mut Self> {
        let graph = self
            .state
            .querier
            .get(
                "something",
                self.inner
                    .current_path
                    .map(|p| p.split('.').collect::<Vec<_>>())
                    .unwrap_or_default(),
            )
            .ok_or(anyhow::anyhow!("graph should've had an item"))?;

        self.inner.graph = Some(graph);

        Ok(self)
    }
}

impl<'a> StatefulWidget for GraphExplorer<'a> {
    type State = GraphExplorerState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let Rect { height, .. } = area;
        let height = height as usize;

        if let Some(graph) = &state.graph {
            if let Ok(graph) = serde_json::to_string_pretty(graph) {
                let lines = graph
                    .split('\n')
                    .take(height)
                    .map(Line::raw)
                    .collect::<Vec<_>>();

                let para = Paragraph::new(lines);

                para.render(area, buf);
            }
        }
    }
}

fn should_quit() -> Result<bool> {
    if event::poll(Duration::from_millis(250)).context("event poll failed")? {
        if let Event::Key(key) = event::read().context("event read failed")? {
            return Ok(KeyCode::Char('q') == key.code);
        }
    }
    Ok(false)
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("failed to enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("unable to enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stdout)).context("creating terminal failed")
}

/// Restore the terminal. This is where you disable raw mode, leave the alternate screen, and show
/// the cursor.
fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("unable to switch to main screen")?;
    terminal.show_cursor().context("unable to show cursor")
}

mod logging;
