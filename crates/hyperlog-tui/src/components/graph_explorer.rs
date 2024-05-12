use anyhow::Result;
use hyperlog_core::log::GraphItem;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use crate::{
    command_parser::Commands,
    commands::{
        batch::BatchCommand, create_section::CreateSectionCommandExt,
        toggle_item::ToggleItemCommandExt, update_graph::UpdateGraphCommandExt, Command,
        IntoCommand,
    },
    components::movement_graph::GraphItemType,
    models::{IOEvent, Msg},
    state::SharedState,
};

use super::{
    movement_graph::{MovementGraph, MovementGraphItem},
    render_graph::summarize::SummarizeRenderGraph,
};

#[derive(Clone, Debug)]
pub enum FilterBy {
    NotDone,
    None,
}

impl Default for FilterBy {
    fn default() -> Self {
        Self::NotDone
    }
}

#[derive(Default, Clone, Debug)]
pub struct DisplayOptions {
    pub filter_by: FilterBy,
}

pub struct GraphExplorer<'a> {
    state: SharedState,

    pub inner: GraphExplorerState<'a>,
}

pub struct GraphExplorerState<'a> {
    root: String,

    current_path: Option<&'a str>,
    current_position: Vec<usize>,

    display_options: DisplayOptions,

    graph: Option<GraphItem>,
}

impl<'a> GraphExplorerState<'a> {
    pub fn update(&mut self, msg: &Msg) -> Option<Command> {
        if let Msg::GraphUpdated(graph_update) = msg {
            match graph_update {
                IOEvent::Initialized => {
                    tracing::trace!("initialized graph");
                }
                IOEvent::Success(graph) => {
                    tracing::trace!("graph updated successfully");
                    self.graph = Some(graph.clone());
                }
                IOEvent::Failure(e) => {
                    tracing::error!("graph update failed: {}", e);
                }
                IOEvent::Optimistic(graph) => {
                    tracing::trace!("graph updated optimistically");
                    self.graph = Some(graph.clone());
                }
            }
        }

        None
    }
}

impl<'a> GraphExplorer<'a> {
    pub fn new(root: String, state: SharedState) -> Self {
        Self {
            state,
            inner: GraphExplorerState::<'a> {
                root,
                current_path: None,
                current_position: Vec::new(),
                graph: None,
                display_options: DisplayOptions::default(),
            },
        }
    }

    pub fn new_update_graph(&self) -> Command {
        self.state.update_graph_command().command(
            &self.inner.root,
            &self
                .inner
                .current_path
                .map(|p| p.split(".").collect_vec())
                .unwrap_or_default(),
        )
    }

    pub async fn update_graph(&mut self) -> Result<&mut Self> {
        let now = std::time::SystemTime::now();

        let graph = self
            .state
            .querier
            .get_async(
                &self.inner.root,
                self.inner
                    .current_path
                    .map(|p| p.split('.').collect::<Vec<_>>())
                    .unwrap_or_default(),
            )
            .await?
            .ok_or(anyhow::anyhow!("graph should've had an item"))?;

        self.inner.graph = Some(graph);

        let elapsed = now.elapsed()?;
        tracing::trace!("Graph.update_graph took: {}nanos", elapsed.as_nanos());

        Ok(self)
    }

    fn linearize_graph(&self) -> Option<MovementGraph> {
        tracing::trace!("current display options: {:?}", self.inner.display_options);
        self.inner
            .graph
            .clone()
            .map(|g| MovementGraph::new(g, &self.inner.display_options))
    }

    /// Will only incrmeent to the next level
    ///
    /// Current: 0.1.0
    /// Available: 0.1.0.[0,1,2]
    /// Choses: 0.1.0.0 else nothing
    pub(crate) fn move_right(&mut self) -> Result<()> {
        if let Some(graph) = self.linearize_graph() {
            tracing::debug!("graph: {:?}", graph);
            let position_items = &self.inner.current_position;

            if let Some(next_item) = graph.next_right(position_items) {
                self.inner.current_position.push(next_item.index);
                tracing::trace!("found next item: {:?}", self.inner.current_position);
            }
        }

        Ok(())
    }

    /// Will only incrmeent to the next level
    ///
    /// Current: 0.1.0
    /// Available: 0.[0,1,2].0
    /// Choses: 0.1 else nothing
    pub(crate) fn move_left(&mut self) -> Result<()> {
        if let Some(last) = self.inner.current_position.pop() {
            tracing::trace!(
                "found last item: {:?}, popped: {}",
                self.inner.current_position,
                last
            );
        }

        Ok(())
    }

    /// Will move up if a sibling exists, or up to the most common sibling between sections
    ///
    /// Current: 0.1.1
    /// Available: 0.[0.[0,1],1.[0,1]]
    /// Chose: 0.1.0 again 0.0 We don't choose a subitem in the next three instead we just find the most common sibling
    pub(crate) fn move_up(&mut self) -> Result<()> {
        if let Some(graph) = self.linearize_graph() {
            let position_items = &self.inner.current_position;

            if let Some(next_item) = graph.next_up(position_items) {
                self.inner.current_position = next_item;
                tracing::trace!("found next up: {:?}", self.inner.current_position)
            }
        }

        Ok(())
    }

    /// Will move down if a sibling exists, or down to the most common sibling between sections
    ///
    /// Current: 0.0.0
    /// Available: 0.[0.[0,1],1.[0,1]]
    /// Chose: 0.0.1 again 0.1
    pub(crate) fn move_down(&mut self) -> Result<()> {
        if let Some(graph) = self.linearize_graph() {
            let position_items = &self.inner.current_position;

            if let Some(next_item) = graph.next_down(position_items) {
                self.inner.current_position = next_item;
                tracing::trace!("found next down: {:?}", self.inner.current_position)
            }
        }

        Ok(())
    }

    pub(crate) fn get_current_path(&self) -> Vec<String> {
        let graph = self.linearize_graph();
        let position_items = &self.inner.current_position;

        if let Some(graph) = graph {
            graph.to_current_path(position_items)
        } else {
            Vec::new()
        }
    }

    fn get_current_item(&self) -> Option<MovementGraphItem> {
        let graph = self.linearize_graph();

        if let Some(graph) = graph {
            graph.get_graph_item(&self.inner.current_position).cloned()
        } else {
            None
        }
    }

    pub fn execute_command(&mut self, command: &Commands) -> anyhow::Result<Option<Command>> {
        let mut batch = BatchCommand::default();

        match command {
            Commands::Archive => {
                if !self.get_current_path().is_empty() {
                    tracing::debug!("archiving path: {:?}", self.get_current_path())
                }
            }
            Commands::CreateSection { name } => {
                if !name.is_empty() {
                    let mut path = self.get_current_path();
                    path.push(name.replace(" ", "-").replace(".", "-"));

                    // self.state
                    //     .commander
                    //     .execute(commander::Command::CreateSection {
                    //         root: self.inner.root.clone(),
                    //         path,
                    //     })?;

                    let cmd = self.state.create_section_command().command(
                        &self.inner.root,
                        &path.iter().map(|i| i.as_str()).collect_vec(),
                    );

                    batch.with(cmd.into_command());
                }
            }
            Commands::Edit => {
                if let Some(item) = self.get_current_item() {
                    let path = self.get_current_path();

                    tracing::debug!(
                        "found item to edit: path: {}, item: {}",
                        path.join("."),
                        item.name
                    );
                    match item.item_type {
                        GraphItemType::Section => {
                            todo!("cannot edit section at the moment")
                        }
                        GraphItemType::Item { .. } => {
                            if let Some(item) = self.state.querier.get(&self.inner.root, path) {
                                if let GraphItem::Item { .. } = item {
                                    return Ok(Some(
                                        Msg::OpenEditItemDialog { item }.into_command(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            Commands::ShowAll => {
                self.inner.display_options.filter_by = FilterBy::None;
            }
            Commands::HideDone => {
                self.inner.display_options.filter_by = FilterBy::NotDone;
            }
            Commands::Test => {
                return Ok(Some(Command::new(|dispatch| {
                    tokio::spawn(async move {
                        dispatch.send(Msg::MoveDown);
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        dispatch.send(Msg::EnterViewMode);
                    });

                    None
                })));
            }

            _ => (),
        }

        //self.update_graph()?;

        Ok(Some(batch.into_command()))
    }

    pub(crate) fn interact(&mut self) -> anyhow::Result<Command> {
        let mut batch = BatchCommand::default();

        if !self.get_current_path().is_empty() {
            tracing::info!("toggling state of items");

            // self.state
            //     .commander
            //     .execute(commander::Command::ToggleItem {
            //         root: self.inner.root.to_string(),
            //         path: self.get_current_path(),
            //     })?;

            let cmd = self.state.toggle_item_command().command(
                &self.inner.root,
                &self
                    .get_current_path()
                    .iter()
                    .map(|i| i.as_str())
                    .collect_vec(),
            );

            batch.with(cmd.into_command());
        }

        //self.update_graph()?;

        Ok(batch.into_command())
    }
}

impl<'a> StatefulWidget for GraphExplorer<'a> {
    type State = GraphExplorerState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let Rect { height, .. } = area;
        let _height = height as usize;

        if let Some(graph) = &state.graph {
            let movement_graph: MovementGraph =
                MovementGraph::new(graph.clone(), &state.display_options);
            let lines = movement_graph.render_graph(&state.current_position);
            let para = Paragraph::new(lines);
            para.render(area, buf);
        }
    }
}
