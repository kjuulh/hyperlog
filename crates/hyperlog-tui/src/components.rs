use std::{collections::HashMap, ops::Deref};

use anyhow::Result;
use hyperlog_core::log::GraphItem;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use crate::state::SharedState;

pub struct GraphExplorer<'a> {
    state: SharedState,

    pub inner: GraphExplorerState<'a>,
}

pub struct GraphExplorerState<'a> {
    current_path: Option<&'a str>,
    current_postition: Vec<usize>,

    graph: Option<GraphItem>,
}

impl GraphExplorer<'_> {
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            inner: GraphExplorerState::<'_> {
                current_path: None,
                current_postition: Vec::new(),
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
            let position_items = &self.inner.current_postition;

            if let Some(next_item) = graph.next_right(position_items) {
                self.inner.current_postition.push(next_item.index);
                tracing::trace!("found next item: {:?}", self.inner.current_postition);
            }
        }

        Ok(())
    }

    /// Will only incrmeent to the next level
    ///
    /// Current: 0.1.0
    /// Available: 0.[0.1.2].0
    /// Choses: 0.1 else nothing
    pub(crate) fn move_left(&mut self) -> Result<()> {
        if let Some(last) = self.inner.current_postition.pop() {
            tracing::trace!(
                "found last item: {:?}, popped: {}",
                self.inner.current_postition,
                last
            );
        }

        Ok(())
    }

    pub(crate) fn move_up(&self) -> Result<()> {
        Ok(())
    }

    pub(crate) fn move_down(&self) -> Result<()> {
        Ok(())
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

#[derive(PartialEq, Eq, Debug, Clone)]
struct MovementGraphItem {
    index: usize,
    name: String,
    values: MovementGraph,
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

    use crate::components::MovementGraphItem;

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
            Box::new(GraphItem::Section(BTreeMap::from([
                (
                    "00".to_string(),
                    Box::new(GraphItem::Section(BTreeMap::new())),
                ),
                (
                    "01".to_string(),
                    Box::new(GraphItem::Section(BTreeMap::from([
                        (
                            "010".to_string(),
                            Box::new(GraphItem::Item {
                                title: "some-title".into(),
                                description: "some-desc".into(),
                                state: ItemState::NotDone,
                            }),
                        ),
                        (
                            "011".to_string(),
                            Box::new(GraphItem::Item {
                                title: "some-title".into(),
                                description: "some-desc".into(),
                                state: ItemState::NotDone,
                            }),
                        ),
                    ]))),
                ),
            ]))),
        )]));

        let actual: MovementGraph = graph.into();

        assert_eq!(
            MovementGraph {
                items: vec![MovementGraphItem {
                    index: 0,
                    name: "0".into(),
                    values: MovementGraph {
                        items: vec![
                            MovementGraphItem {
                                index: 0,
                                name: "00".into(),
                                values: MovementGraph::default()
                            },
                            MovementGraphItem {
                                index: 1,
                                name: "01".into(),
                                values: MovementGraph {
                                    items: vec![
                                        MovementGraphItem {
                                            index: 0,
                                            name: "010".into(),
                                            values: MovementGraph::default(),
                                        },
                                        MovementGraphItem {
                                            index: 1,
                                            name: "011".into(),
                                            values: MovementGraph::default(),
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
}
