use std::{io::Stdout, time::Duration};

use anyhow::{Context, Result};
use app::{render_app, App};
use components::GraphExplorer;
use crossterm::event::{self, Event, KeyCode};
use hyperlog_core::state::State;
use models::Msg;
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{state::SharedState, terminal::TerminalInstance};

pub mod models;

pub(crate) mod app;
pub(crate) mod components;
pub(crate) mod state;

mod logging;
mod terminal;

pub async fn execute(state: State) -> Result<()> {
    tracing::debug!("starting hyperlog tui");

    logging::initialize_panic_handler()?;
    logging::initialize_logging()?;

    let state = SharedState::from(state);

    let mut terminal = TerminalInstance::new()?;
    run(&mut terminal, state).context("app loop failed")?;

    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, state: SharedState) -> Result<()> {
    let mut graph_explorer = GraphExplorer::new(state.clone());
    graph_explorer.update_graph()?;

    let mut app = App::new(state.clone(), graph_explorer);

    loop {
        terminal.draw(|f| render_app(f, &mut app))?;

        if update(terminal, &mut app)?.should_quit() {
            break;
        }
    }
    Ok(())
}

pub struct UpdateConclusion(bool);

impl UpdateConclusion {
    pub fn new(should_quit: bool) -> Self {
        Self(should_quit)
    }

    pub fn should_quit(self) -> bool {
        self.0
    }
}

fn update(
    _terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<UpdateConclusion> {
    if event::poll(Duration::from_millis(250)).context("event poll failed")? {
        if let Event::Key(key) = event::read().context("event read failed")? {
            match key.code {
                KeyCode::Char('q') => return Ok(UpdateConclusion::new(true)),
                KeyCode::Char('l') => {
                    app.update(Msg::MoveRight)?;
                }
                KeyCode::Char('h') => {
                    app.update(Msg::MoveLeft)?;
                }
                KeyCode::Char('j') => {
                    app.update(Msg::MoveDown)?;
                }
                KeyCode::Char('k') => {
                    app.update(Msg::MoveUp)?;
                }
                _ => {}
            }
        }
    }

    Ok(UpdateConclusion::new(false))
}
