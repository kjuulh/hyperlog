use std::ops::Deref;

use anyhow::Result;
use hyperlog_core::log::{GraphItem, ItemState};
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use crate::{command_parser::Commands, models::Msg, state::SharedState};

pub struct GraphExplorer<'a> {
    state: SharedState,

    pub inner: GraphExplorerState<'a>,
}

pub struct GraphExplorerState<'a> {
    root: String,

    current_path: Option<&'a str>,
    current_position: Vec<usize>,

    graph: Option<GraphItem>,
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
            },
        }
    }

    pub fn update_graph(&mut self) -> Result<&mut Self> {
        let now = std::time::SystemTime::now();

        let graph = self
            .state
            .querier
            .get(
                &self.inner.root,
                self.inner
                    .current_path
                    .map(|p| p.split('.').collect::<Vec<_>>())
                    .unwrap_or_default(),
            )
            .ok_or(anyhow::anyhow!("graph should've had an item"))?;

        self.inner.graph = Some(graph);

        let elapsed = now.elapsed()?;
        tracing::trace!("Graph.update_graph took: {}nanos", elapsed.as_nanos());

        Ok(self)
    }

    fn linearize_graph(&self) -> Option<MovementGraph> {
        self.inner.graph.clone().map(|g| g.into())
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

    pub fn execute_command(&mut self, command: &Commands) -> anyhow::Result<Option<Msg>> {
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

                    self.state.commander.execute(
                        hyperlog_core::commander::Command::CreateSection {
                            root: self.inner.root.clone(),
                            path,
                        },
                    )?;
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
                                    return Ok(Some(Msg::OpenEditItemDialog { item }));
                                }
                            }
                        }
                    }
                }
            }
            _ => (),
        }

        self.update_graph()?;

        Ok(None)
    }

    pub(crate) fn interact(&mut self) -> anyhow::Result<()> {
        if !self.get_current_path().is_empty() {
            tracing::info!("toggling state of items");

            self.state
                .commander
                .execute(hyperlog_core::commander::Command::ToggleItem {
                    root: self.inner.root.to_string(),
                    path: self.get_current_path(),
                })?;
        }

        self.update_graph()?;

        Ok(())
    }
}

trait RenderGraph {
    fn render_graph(&self, items: &[usize]) -> Vec<Line>;
    fn render_graph_spans(&self, items: &[usize]) -> Vec<Vec<Span>>;
}

impl RenderGraph for MovementGraph {
    /// render_graph takes each level of items, renders them, and finally renders a strongly set selector for the current item the user is on
    /// This is done from buttom up, and composed via. string padding
    fn render_graph(&self, items: &[usize]) -> Vec<Line> {
        // Gets the inner content of the strings

        let mut lines = Vec::new();

        for item in &self.items {
            let prefix = match item.item_type {
                GraphItemType::Section => "- ",
                GraphItemType::Item { done } => {
                    if done {
                        "- [x]"
                    } else {
                        "- [ ]"
                    }
                }
            };

            match items.split_first().map(|(first, rest)| {
                if item.index == *first {
                    (true, rest)
                } else {
                    (false, rest)
                }
            }) {
                Some((true, rest)) => {
                    if rest.is_empty() {
                        lines.push(
                            Line::raw(format!("{} {}", prefix, item.name))
                                .style(Style::new().bold().white()),
                        );
                    } else {
                        lines.push(
                            Line::raw(format!("{} {}", prefix, item.name))
                                .patch_style(Style::new().dark_gray()),
                        );
                    }

                    lines.push("".into());

                    let embedded_sections = item.values.render_graph_spans(rest);
                    for section in &embedded_sections {
                        let mut line = vec![Span::raw(" ".repeat(4))];
                        line.extend_from_slice(section);
                        lines.push(Line::from(line));
                    }
                }
                _ => {
                    lines.push(
                        Line::raw(format!("{} {}", prefix, item.name))
                            .patch_style(Style::new().dark_gray()),
                    );

                    lines.push("".into());

                    let embedded_sections = item.values.render_graph_spans(&[]);
                    for section in &embedded_sections {
                        let mut line = vec![Span::raw(" ".repeat(4))];
                        line.extend_from_slice(section);
                        lines.push(Line::from(line));
                    }
                }
            }
        }

        lines
    }

    fn render_graph_spans(&self, items: &[usize]) -> Vec<Vec<Span>> {
        let mut lines = Vec::new();

        for item in &self.items {
            let prefix = match item.item_type {
                GraphItemType::Section => "-",
                GraphItemType::Item { done } => {
                    if done {
                        "- [x]"
                    } else {
                        "- [ ]"
                    }
                }
            };
            match items.split_first().map(|(first, rest)| {
                if item.index == *first {
                    (true, rest)
                } else {
                    (false, rest)
                }
            }) {
                Some((true, rest)) => {
                    let mut line = Vec::new();
                    if rest.is_empty() {
                        line.push(
                            Span::raw(format!("{} {}", prefix, item.name))
                                .style(Style::new().bold().white()),
                        );
                    } else {
                        line.push(
                            Span::raw(format!("{} {}", prefix, item.name))
                                .patch_style(Style::new().dark_gray()),
                        );
                    }

                    lines.push(line);
                    lines.push(vec!["".into()]);

                    let embedded_sections = item.values.render_graph_spans(rest);
                    for section in &embedded_sections {
                        let mut line = vec![Span::raw(" ".repeat(4))];
                        line.extend_from_slice(section);
                        lines.push(line);
                    }
                }
                _ => {
                    lines.push(vec![Span::raw(format!("{prefix} {}", item.name))
                        .patch_style(Style::new().dark_gray())]);

                    lines.push(vec!["".into()]);

                    let embedded_sections = item.values.render_graph_spans(&[]);
                    for section in &embedded_sections {
                        let mut line = vec![Span::raw(" ".repeat(4))];
                        line.extend_from_slice(section);
                        lines.push(line);
                    }
                }
            }
        }

        lines
    }
}

impl<'a> StatefulWidget for GraphExplorer<'a> {
    type State = GraphExplorerState<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let Rect { height, .. } = area;
        let _height = height as usize;

        if let Some(graph) = &state.graph {
            let movement_graph: MovementGraph = graph.clone().into();
            let lines = movement_graph.render_graph(&state.current_position);
            let para = Paragraph::new(lines);
            para.render(area, buf);
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum GraphItemType {
    Section,
    Item { done: bool },
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct MovementGraphItem {
    index: usize,
    name: String,
    values: MovementGraph,

    item_type: GraphItemType,
}

#[derive(Default, PartialEq, Eq, Debug, Clone)]
struct MovementGraph {
    items: Vec<MovementGraphItem>,
}

impl MovementGraph {
    fn next_right(&self, items: &[usize]) -> Option<MovementGraphItem> {
        match items.split_first() {
            Some((current_index, rest)) => match self.items.get(*current_index) {
                Some(next_item) => next_item.values.next_right(rest),
                None => None,
            },
            None => self.items.first().cloned(),
        }
    }

    fn next_up(&self, items: &[usize]) -> Option<Vec<usize>> {
        match items.split_last() {
            Some((0, _)) => None,
            Some((current_index, rest)) => {
                let mut vec = rest.to_vec();
                vec.push(current_index - 1);

                Some(vec)
            }
            // May need to reduce this to an Some(Vec::default()) instead
            //None => Some(self.items.iter().map(|i| i.index).collect_vec()),
            None => None,
        }
    }

    fn next_down(&self, items: &[usize]) -> Option<Vec<usize>> {
        match items.split_last() {
            Some((current_index, rest)) => {
                if let Some(current_item) = self.get_graph(rest) {
                    if *current_index + 1 < current_item.items.len() {
                        let mut vec = rest.to_vec();
                        vec.push(current_index + 1);

                        Some(vec)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            // May need to reduce this to an Some(Vec::default()) instead
            //None => Some(self.items.iter().map(|i| i.index).collect_vec()),
            None => None,
        }
    }

    fn get_graph(&self, items: &[usize]) -> Option<&MovementGraph> {
        match items.split_first() {
            Some((first, rest)) => match self.items.get(*first).map(|s| &s.values) {
                Some(next_graph) => next_graph.get_graph(rest),
                None => Some(self),
            },
            None => Some(self),
        }
    }

    fn get_graph_item(&self, items: &[usize]) -> Option<&MovementGraphItem> {
        match items.split_first() {
            Some((first, rest)) => match self.items.get(*first) {
                Some(next_graph) => match next_graph.values.get_graph_item(rest) {
                    Some(graph_item) => Some(graph_item),
                    None => Some(next_graph),
                },
                None => None,
            },
            None => None,
        }
    }

    fn to_current_path(&self, position_items: &[usize]) -> Vec<String> {
        match position_items.split_first() {
            Some((first, rest)) => match self.items.get(*first) {
                Some(item) => {
                    let mut current = vec![item.name.clone()];
                    let mut next = item.values.to_current_path(rest);
                    current.append(&mut next);

                    current
                }
                None => Vec::new(),
            },
            None => Vec::new(),
        }
    }
}

impl From<Box<GraphItem>> for MovementGraph {
    fn from(value: Box<GraphItem>) -> Self {
        value.deref().clone().into()
    }
}

impl From<GraphItem> for MovementGraph {
    fn from(value: GraphItem) -> Self {
        let mut graph = MovementGraph::default();

        match value {
            GraphItem::User(sections) | GraphItem::Section(sections) => {
                let graph_items = sections
                    .iter()
                    .sorted_by(|(a, _), (b, _)| Ord::cmp(a, b))
                    .enumerate()
                    .map(|(i, (key, value))| MovementGraphItem {
                        index: i,
                        name: key.clone(),
                        values: value.clone().into(),
                        item_type: match value {
                            GraphItem::User(_) => GraphItemType::Section,
                            GraphItem::Section(_) => GraphItemType::Section,
                            GraphItem::Item { state, .. } => GraphItemType::Item {
                                done: matches!(state, ItemState::Done),
                            },
                        },
                    })
                    .collect::<Vec<_>>();

                graph.items = graph_items;
            }
            GraphItem::Item { .. } => {}
        }

        graph
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use hyperlog_core::log::{GraphItem, ItemState};
    use similar_asserts::assert_eq;

    use crate::components::{GraphItemType, MovementGraphItem};

    use super::MovementGraph;

    /// Lets say we've got a graph
    /// ```json
    ///  {
    ///    "type": "user",
    ///    "something": {
    ///      "type": "section",
    ///      "something": {
    ///        "type": "section",
    ///        "something-else": {
    ///          "type": "section",
    ///          "blabla": {
    ///            "type": "section"
    ///          }
    ///        }
    ///      }
    ///    }
    ///  }
    /// ```
    /// We can get something out like
    /// [
    ///   0: {key: something, values: [
    ///         0: {key: something, values: [
    ///               ...
    ///            ]}
    ///      ]}    
    /// ]
    #[test]
    fn test_can_transform_to_movement_graph() {
        let graph = GraphItem::User(BTreeMap::from([(
            "0".to_string(),
            GraphItem::Section(BTreeMap::from([
                ("00".to_string(), GraphItem::Section(BTreeMap::new())),
                (
                    "01".to_string(),
                    GraphItem::Section(BTreeMap::from([
                        (
                            "010".to_string(),
                            GraphItem::Item {
                                title: "some-title".into(),
                                description: "some-desc".into(),
                                state: ItemState::NotDone,
                            },
                        ),
                        (
                            "011".to_string(),
                            GraphItem::Item {
                                title: "some-title".into(),
                                description: "some-desc".into(),
                                state: ItemState::NotDone,
                            },
                        ),
                    ])),
                ),
            ])),
        )]));

        let actual: MovementGraph = graph.into();

        assert_eq!(
            MovementGraph {
                items: vec![MovementGraphItem {
                    index: 0,
                    name: "0".into(),
                    item_type: GraphItemType::Section,
                    values: MovementGraph {
                        items: vec![
                            MovementGraphItem {
                                index: 0,
                                name: "00".into(),
                                values: MovementGraph::default(),
                                item_type: GraphItemType::Section,
                            },
                            MovementGraphItem {
                                index: 1,
                                name: "01".into(),
                                item_type: GraphItemType::Section,
                                values: MovementGraph {
                                    items: vec![
                                        MovementGraphItem {
                                            index: 0,
                                            name: "010".into(),
                                            values: MovementGraph::default(),
                                            item_type: GraphItemType::Item { done: false },
                                        },
                                        MovementGraphItem {
                                            index: 1,
                                            name: "011".into(),
                                            values: MovementGraph::default(),
                                            item_type: GraphItemType::Item { done: false },
                                        },
                                    ]
                                }
                            },
                        ]
                    }
                }]
            },
            actual
        );
    }

    #[test]
    fn test_get_graph_item() -> anyhow::Result<()> {
        let graph = MovementGraph {
            items: vec![
                MovementGraphItem {
                    index: 0,
                    name: "0".into(),
                    item_type: GraphItemType::Section,
                    values: MovementGraph {
                        items: vec![
                            MovementGraphItem {
                                index: 0,
                                name: "0".into(),
                                values: MovementGraph::default(),
                                item_type: GraphItemType::Section,
                            },
                            MovementGraphItem {
                                index: 1,
                                name: "0".into(),
                                values: MovementGraph::default(),
                                item_type: GraphItemType::Section,
                            },
                        ],
                    },
                },
                MovementGraphItem {
                    index: 1,
                    name: "0".into(),
                    item_type: GraphItemType::Section,
                    values: MovementGraph {
                        items: vec![
                            MovementGraphItem {
                                index: 0,
                                name: "0".into(),
                                values: MovementGraph::default(),
                                item_type: GraphItemType::Section,
                            },
                            MovementGraphItem {
                                index: 1,
                                name: "0".into(),
                                values: MovementGraph::default(),
                                item_type: GraphItemType::Section,
                            },
                            MovementGraphItem {
                                index: 2,
                                name: "0".into(),
                                values: MovementGraph::default(),
                                item_type: GraphItemType::Section,
                            },
                        ],
                    },
                },
                MovementGraphItem {
                    index: 2,
                    name: "0".into(),
                    item_type: GraphItemType::Section,
                    values: MovementGraph {
                        items: vec![
                            MovementGraphItem {
                                index: 0,
                                name: "0".into(),
                                values: MovementGraph::default(),
                                item_type: GraphItemType::Section,
                            },
                            MovementGraphItem {
                                index: 1,
                                name: "0".into(),
                                values: MovementGraph::default(),
                                item_type: GraphItemType::Section,
                            },
                        ],
                    },
                },
            ],
        };

        let actual_default = graph.get_graph(&[]);
        assert_eq!(Some(&graph), actual_default);

        let actual_first = graph.get_graph(&[0]);
        assert_eq!(graph.items.first().map(|i| &i.values), actual_first);

        let actual_second = graph.get_graph(&[1]);
        assert_eq!(graph.items.get(1).map(|i| &i.values), actual_second);

        let actual_nested = graph.get_graph(&[0, 0]);
        assert_eq!(
            graph
                .items
                .first()
                .and_then(|i| i.values.items.first())
                .map(|i| &i.values),
            actual_nested
        );

        let actual_nested = graph.get_graph(&[0, 1]);
        assert_eq!(
            graph
                .items
                .first()
                .and_then(|i| i.values.items.get(1))
                .map(|i| &i.values),
            actual_nested
        );

        let actual_nested = graph.get_graph(&[1, 2]);
        assert_eq!(
            graph
                .items
                .get(1)
                .and_then(|i| i.values.items.get(2))
                .map(|i| &i.values),
            actual_nested
        );

        Ok(())
    }

    #[test]
    fn can_next_down() -> anyhow::Result<()> {
        let graph = MovementGraph {
            items: vec![
                MovementGraphItem {
                    index: 0,
                    name: "0".into(),
                    item_type: GraphItemType::Section,
                    values: MovementGraph {
                        items: vec![MovementGraphItem {
                            index: 0,
                            name: "0".into(),
                            item_type: GraphItemType::Section,
                            values: MovementGraph::default(),
                        }],
                    },
                },
                MovementGraphItem {
                    index: 1,
                    name: "1".into(),
                    item_type: GraphItemType::Section,
                    values: MovementGraph {
                        items: vec![
                            MovementGraphItem {
                                index: 0,
                                name: "0".into(),
                                item_type: GraphItemType::Section,
                                values: MovementGraph::default(),
                            },
                            MovementGraphItem {
                                index: 1,
                                name: "1".into(),
                                item_type: GraphItemType::Section,
                                values: MovementGraph::default(),
                            },
                        ],
                    },
                },
                MovementGraphItem {
                    index: 2,
                    name: "2".into(),
                    item_type: GraphItemType::Section,
                    values: MovementGraph {
                        items: vec![
                            MovementGraphItem {
                                index: 0,
                                name: "0".into(),
                                item_type: GraphItemType::Section,
                                values: MovementGraph::default(),
                            },
                            MovementGraphItem {
                                index: 1,
                                name: "1".into(),
                                item_type: GraphItemType::Section,
                                values: MovementGraph::default(),
                            },
                            MovementGraphItem {
                                index: 2,
                                name: "2".into(),
                                item_type: GraphItemType::Section,
                                values: MovementGraph::default(),
                            },
                        ],
                    },
                },
            ],
        };

        let actual = graph.next_down(&[]);
        assert_eq!(None, actual);

        let actual = graph.next_down(&[0]);
        assert_eq!(Some(vec![1]), actual);

        let actual = graph.next_down(&[1]);
        assert_eq!(Some(vec![2]), actual);

        let actual = graph.next_down(&[2]);
        assert_eq!(None, actual);

        let graph = MovementGraph {
            items: vec![
                MovementGraphItem {
                    index: 0,
                    name: "other".into(),
                    item_type: GraphItemType::Section,
                    values: MovementGraph {
                        items: vec![MovementGraphItem {
                            index: 0,
                            name: "other".into(),
                            item_type: GraphItemType::Section,
                            values: MovementGraph {
                                items: vec![MovementGraphItem {
                                    index: 0,
                                    name: "other".into(),
                                    item_type: GraphItemType::Section,
                                    values: MovementGraph { items: vec![] },
                                }],
                            },
                        }],
                    },
                },
                MovementGraphItem {
                    index: 1,
                    name: "some".into(),
                    item_type: GraphItemType::Section,
                    values: MovementGraph { items: vec![] },
                },
                MovementGraphItem {
                    index: 2,
                    name: "something".into(),
                    item_type: GraphItemType::Section,
                    values: MovementGraph {
                        items: vec![
                            MovementGraphItem {
                                index: 0,
                                name: "else".into(),
                                item_type: GraphItemType::Section,
                                values: MovementGraph { items: vec![] },
                            },
                            MovementGraphItem {
                                index: 1,
                                name: "third".into(),
                                item_type: GraphItemType::Section,
                                values: MovementGraph { items: vec![] },
                            },
                        ],
                    },
                },
            ],
        };

        let actual = graph.next_down(&[0]);
        assert_eq!(Some(vec![1]), actual);

        Ok(())
    }
}
