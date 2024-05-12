#![feature(map_try_insert)]
#![feature(fn_traits)]
#![feature(let_chains)]

use std::io::Stdout;

use anyhow::{Context, Result};
use app::{render_app, App};
use commands::{Dispatch, IntoCommand, Receiver};
use components::graph_explorer::GraphExplorer;
use core_state::State;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use futures::{FutureExt, StreamExt};
use models::{EditMsg, Msg};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{state::SharedState, terminal::TerminalInstance};

pub mod models;

pub(crate) mod app;
pub(crate) mod command_parser;
pub(crate) mod commands;
pub(crate) mod components;

pub mod commander;
pub mod core_state;
pub mod shared_engine;
pub mod state;

mod engine;
mod events;
mod querier;
mod storage;

mod logging;
mod terminal;

pub async fn execute(state: State) -> Result<()> {
    tracing::debug!("starting hyperlog tui");

    logging::initialize_panic_handler()?;
    logging::initialize_logging()?;

    let state = SharedState::from(state);

    let mut terminal = TerminalInstance::new()?;
    run(&mut terminal, state).await.context("app loop failed")?;

    Ok(())
}

async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, state: SharedState) -> Result<()> {
    let root = match state.querier.get_available_roots_async().await? {
        // TODO: maybe present choose root screen
        Some(roots) => roots.first().cloned().unwrap(),
        None => {
            // TODO: present create root screen
            anyhow::bail!("no valid root available\nPlease run:\n\n$ hyperlog create-root --name <your-username>");
        }
    };

    let mut graph_explorer = GraphExplorer::new(root.clone(), state.clone());
    graph_explorer.update_graph().await?;

    let mut app = App::new(&root, state.clone(), graph_explorer);
    let (dispatch, mut receiver) = commands::create_dispatch();
    let mut event_stream = crossterm::event::EventStream::new();

    loop {
        terminal.draw(|f| render_app(f, &mut app))?;

        if update(
            terminal,
            &mut app,
            &dispatch,
            &mut receiver,
            &mut event_stream,
        )
        .await?
        .should_quit()
        {
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

async fn update<'a>(
    _terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App<'a>,
    dispatch: &Dispatch,
    receiver: &mut Receiver,
    event_stream: &mut crossterm::event::EventStream,
) -> Result<UpdateConclusion> {
    let cross_event = event_stream.next().fuse();

    let mut handle_key_event = |maybe_event| -> anyhow::Result<UpdateConclusion> {
        match maybe_event {
            Some(Ok(e)) => {
                if let Event::Key(key) = e
                    && key.kind == KeyEventKind::Press
                {
                    let mut cmd = match &app.mode {
                        app::Mode::View => match key.code {
                            KeyCode::Enter => app.update(Msg::Interact)?,
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
                        let msg = cmd.into_command().execute(dispatch.clone());
                        match msg {
                            Some(msg) => {
                                if let Msg::QuitApp = msg {
                                    return Ok(UpdateConclusion(true));
                                }

                                cmd = app.update(msg)?;
                            }
                            None => break,
                        }
                    }
                }
            }
            Some(Err(e)) => {
                tracing::warn!("failed to send event: {}", e);
            }
            None => {}
        }

        Ok(UpdateConclusion(false))
    };

    tokio::select! {
        maybe_event = cross_event => {
            let conclusion = handle_key_event(maybe_event)?;

            return Ok(conclusion)
        },

        msg = receiver.next() => {
            if let Some(msg) = msg {
                if let Msg::QuitApp = msg {
                    return Ok(UpdateConclusion(true));
                }

                let mut cmd = app.update(msg)?;

                loop {
                    let msg = cmd.into_command().execute(dispatch.clone());
                    match msg {
                        Some(msg) => {
                            if let Msg::QuitApp = msg {
                                return Ok(UpdateConclusion(true));
                            }

                            cmd = app.update(msg)?;
                        }
                        None => break,
                    }
                }
            }
        }
    }

    Ok(UpdateConclusion::new(false))
}
