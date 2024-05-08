#![feature(fn_traits)]

use std::{io::Stdout, time::Duration};

use anyhow::{Context, Result};
use app::{render_app, App};
use commands::IntoCommand;
use components::GraphExplorer;
use crossterm::event::{self, Event, KeyCode};
use hyperlog_core::state::State;
use models::{EditMsg, Msg};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{state::SharedState, terminal::TerminalInstance};

pub mod models;

pub(crate) mod app;
pub(crate) mod commands;
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
            let mut cmd = match &app.mode {
                app::Mode::View => match key.code {
                    KeyCode::Char('q') => return Ok(UpdateConclusion::new(true)),
                    KeyCode::Char('l') => app.update(Msg::MoveRight)?,
                    KeyCode::Char('h') => app.update(Msg::MoveLeft)?,
                    KeyCode::Char('j') => app.update(Msg::MoveDown)?,
                    KeyCode::Char('k') => app.update(Msg::MoveUp)?,
                    KeyCode::Char('a') => {
                        // TODO: batch commands
                        app.update(Msg::OpenCreateItemDialog)?;
                        app.update(Msg::EnterInsertMode)?
                    }
                    KeyCode::Char('i') => app.update(Msg::EnterInsertMode)?,
                    KeyCode::Char(':') => app.update(Msg::EnterCommandMode)?,
                    _ => return Ok(UpdateConclusion(false)),
                },

                app::Mode::Command | app::Mode::Insert => match key.code {
                    KeyCode::Backspace => app.update(Msg::Edit(EditMsg::Delete))?,
                    KeyCode::Enter => app.update(Msg::Edit(EditMsg::InsertNewLine))?,
                    KeyCode::Tab => app.update(Msg::Edit(EditMsg::InsertTab))?,
                    KeyCode::Delete => app.update(Msg::Edit(EditMsg::DeleteNext))?,
                    KeyCode::Char(c) => app.update(Msg::Edit(EditMsg::InsertChar(c)))?,
                    KeyCode::Left => app.update(Msg::Edit(EditMsg::MoveLeft))?,
                    KeyCode::Right => app.update(Msg::Edit(EditMsg::MoveRight))?,
                    KeyCode::Esc => app.update(Msg::EnterViewMode)?,
                    _ => return Ok(UpdateConclusion(false)),
                },
            };

            loop {
                let msg = cmd.into_command().execute();
                match msg {
                    Some(msg) => {
                        cmd = app.update(msg)?;
                    }
                    None => break,
                }
            }
        }
    }

    Ok(UpdateConclusion::new(false))
}
