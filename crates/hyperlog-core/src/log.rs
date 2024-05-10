use std::{
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
pub enum ItemState {
    #[serde(rename = "not-done")]
    NotDone,
    #[serde(rename = "done")]
    Done,
}

impl Default for ItemState {
    fn default() -> Self {
        Self::NotDone
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Debug)]
#[serde(tag = "type")]
pub enum GraphItem {
    #[serde(rename = "user")]
    User(BTreeMap<String, GraphItem>),
    #[serde(rename = "section")]
    Section(BTreeMap<String, GraphItem>),
    #[serde(rename = "item")]
    Item {
        title: String,
        description: String,
        state: ItemState,
    },
}

impl GraphItem {
    pub fn get(&self, path: &[&str]) -> Option<&GraphItem> {
        match path.split_first() {
            Some((first, rest)) => match self {
                GraphItem::User(section) | GraphItem::Section(section) => {
                    section.get(*first)?.get(rest)
                }
                GraphItem::Item { .. } => None,
            },
            None => Some(self),
        }
    }

    pub fn get_mut(&mut self, path: &[&str]) -> Option<&mut GraphItem> {
        match path.split_first() {
            Some((first, rest)) => match self {
                GraphItem::User(section) | GraphItem::Section(section) => {
                    section.get_mut(*first)?.get_mut(rest)
                }
                GraphItem::Item { .. } => None,
            },
            None => Some(self),
        }
    }

    pub fn take(&mut self, path: &[&str]) -> Option<GraphItem> {
        match path.split_first() {
            Some((first, rest)) => match self {
                GraphItem::User(section) | GraphItem::Section(section) => {
                    if rest.is_empty() {
                        section.remove(*first)
                    } else {
                        section.get_mut(*first)?.take(rest)
                    }
                }
                GraphItem::Item { .. } => None,
            },
            None => None,
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Graph(BTreeMap<String, GraphItem>);

impl Deref for Graph {
    type Target = BTreeMap<String, GraphItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Graph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use crate::log::{GraphItem, ItemState};

    use super::Graph;

    #[test]
    fn some_test() {
        let test_graph = r#"{
  "kjuulh": {
    "type": "user"
  }
}"#;

        let graph: Graph = serde_json::from_str(test_graph).unwrap();

        let mut expected = Graph::default();

        let user = BTreeMap::new();

        expected.insert("kjuulh".into(), GraphItem::User(user));
        similar_asserts::assert_eq!(expected, graph);
    }

    #[test]
    fn some_user_test() {
        let test_graph = r#"{
  "kjuulh": {
    "type": "user",
    "some-project": {
      "type": "section"
    }
  }
}"#;

        let graph: Graph = serde_json::from_str(test_graph).unwrap();

        let mut expected = Graph::default();
        let mut user = BTreeMap::new();
        user.insert(
            "some-project".into(),
            GraphItem::Section(BTreeMap::default()),
        );

        expected.insert("kjuulh".into(), GraphItem::User(user));

        similar_asserts::assert_eq!(expected, graph);
    }

    #[test]
    fn some_section_test() {
        let test_graph = r#"{
  "kjuulh": {
    "type": "user",
    "some-project": {
      "type": "section",
      "some-nested-project": {
        "type": "section"
      }
    }
  }
}"#;

        let graph: Graph = serde_json::from_str(test_graph).unwrap();

        let mut expected = Graph::default();

        let mut some_project = BTreeMap::default();
        some_project.insert(
            "some-nested-project".into(),
            GraphItem::Section(BTreeMap::default()),
        );
        let mut user = BTreeMap::new();
        user.insert("some-project".into(), GraphItem::Section(some_project));

        expected.insert("kjuulh".into(), GraphItem::User(user));

        similar_asserts::assert_eq!(expected, graph);
    }

    #[test]
    fn some_item_test() {
        let test_graph = r#"{
  "kjuulh": {
    "type": "user",
    "some-project": {
      "type": "section",
      "some-nested-project": {
        "type": "section",
        "some-todo": {
          "type": "item",
          "title": "some title",
          "description": "some description",
          "state": "not-done"
        }
      }
    }
  }
}"#;

        let graph: Graph = serde_json::from_str(test_graph).unwrap();

        let mut expected = Graph::default();

        let mut nested_project = BTreeMap::default();
        nested_project.insert(
            "some-todo".into(),
            GraphItem::Item {
                title: "some title".into(),
                description: "some description".into(),
                state: ItemState::NotDone,
            },
        );

        let mut some_project = BTreeMap::default();
        some_project.insert(
            "some-nested-project".into(),
            GraphItem::Section(nested_project),
        );
        let mut user = BTreeMap::new();
        user.insert("some-project".into(), GraphItem::Section(some_project));

        expected.insert("kjuulh".into(), GraphItem::User(user));

        similar_asserts::assert_eq!(
            serde_json::to_string_pretty(&expected).unwrap(),
            serde_json::to_string_pretty(&graph).unwrap()
        );
    }
}
