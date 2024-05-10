use std::ops::Deref;

use hyperlog_core::log::{GraphItem, ItemState};
use itertools::Itertools;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum GraphItemType {
    Section,
    Item { done: bool },
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct MovementGraphItem {
    pub index: usize,
    pub name: String,
    pub values: MovementGraph,

    pub item_type: GraphItemType,
}

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct MovementGraph {
    pub items: Vec<MovementGraphItem>,
}

impl MovementGraph {
    pub fn next_right(&self, items: &[usize]) -> Option<MovementGraphItem> {
        match items.split_first() {
            Some((current_index, rest)) => match self.items.get(*current_index) {
                Some(next_item) => next_item.values.next_right(rest),
                None => None,
            },
            None => self.items.first().cloned(),
        }
    }

    pub fn next_up(&self, items: &[usize]) -> Option<Vec<usize>> {
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

    pub fn next_down(&self, items: &[usize]) -> Option<Vec<usize>> {
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

    pub fn get_graph_item(&self, items: &[usize]) -> Option<&MovementGraphItem> {
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

    pub fn to_current_path(&self, position_items: &[usize]) -> Vec<String> {
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

    use crate::components::movement_graph::{GraphItemType, MovementGraphItem};

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
