use std::{collections::BTreeMap, fmt::Display};

use anyhow::anyhow;

use crate::log::{Graph, GraphItem};

#[derive(Default)]
pub struct Engine {
    graph: Graph,
}

impl Engine {
    pub fn create_root(&mut self, root: &str) -> anyhow::Result<()> {
        self.graph
            .try_insert(root.to_string(), GraphItem::User(BTreeMap::default()))
            .map_err(|_| anyhow!("entry was already found, aborting"))?;

        Ok(())
    }

    pub fn create(&mut self, root: &str, path: &[&str], item: GraphItem) -> anyhow::Result<()> {
        let graph = &mut self.graph;

        let (last, items) = path.split_last().ok_or(anyhow!(
            "path cannot be empty, must contain at least one item"
        ))?;

        let root = graph
            .get_mut(root)
            .ok_or(anyhow!("root was missing a user, aborting"))?;

        let mut current_item = root;
        for section in items {
            match current_item {
                GraphItem::User(u) => match u.get_mut(section.to_owned()) {
                    Some(graph_item) => {
                        current_item = graph_item.as_mut();
                    }
                    None => anyhow::bail!("path: {} section was not found", section),
                },
                GraphItem::Item { .. } => anyhow::bail!("path: {} was already found", section),
                GraphItem::Section(s) => match s.get_mut(section.to_owned()) {
                    Some(graph_item) => {
                        current_item = graph_item;
                    }
                    None => anyhow::bail!("path: {} section was not found", section),
                },
            }
        }

        match current_item {
            GraphItem::User(u) => {
                u.insert(last.to_string(), Box::new(item));
            }
            GraphItem::Section(s) => {
                s.insert(last.to_string(), Box::new(item));
            }
            GraphItem::Item { .. } => anyhow::bail!("cannot insert an item into an item"),
        }

        Ok(())
    }

    pub fn get(&self, root: &str, path: &[&str]) -> Option<&GraphItem> {
        let root = self.graph.get(root)?;

        root.get(path)
    }

    pub fn get_mut(&mut self, root: &str, path: &[&str]) -> Option<&mut GraphItem> {
        let root = self.graph.get_mut(root)?;

        root.get_mut(path)
    }

    pub fn take(&mut self, root: &str, path: &[&str]) -> Option<GraphItem> {
        let root = self.graph.get_mut(root)?;

        root.take(path)
    }

    pub fn section_move(
        &mut self,
        root: &str,
        src_path: &[&str],
        dest_path: &[&str],
    ) -> anyhow::Result<()> {
        let src = self
            .take(root, src_path)
            .ok_or(anyhow!("failed to find source path"))?;

        let dest = self
            .get_mut(root, dest_path)
            .ok_or(anyhow!("failed to find destination"))?;

        let src_item = src_path
            .last()
            .ok_or(anyhow!("src path must have at least one item"))?;

        match dest {
            GraphItem::User(u) => {
                u.try_insert(src_item.to_string(), Box::new(src))
                    .map_err(|_e| anyhow!("key was already found, aborting: {}", src_item))?;
            }
            GraphItem::Section(s) => {
                s.try_insert(src_item.to_string(), Box::new(src))
                    .map_err(|_e| anyhow!("key was already found, aborting: {}", src_item))?;
            }
            GraphItem::Item { .. } => {
                anyhow::bail!("failed to insert src at item, item doesn't support arbitrary items")
            }
        }

        Ok(())
    }

    pub fn delete(&mut self, root: &str, path: &[&str]) -> anyhow::Result<()> {
        self.take(root, path)
            .map(|_| ())
            .ok_or(anyhow!("item was not found"))
    }
}

impl Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = serde_json::to_string_pretty(&self.graph).unwrap();
        f.write_str(&output)
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use similar_asserts::assert_eq;

    use crate::log::{GraphItem, ItemState};

    use super::Engine;

    #[test]
    fn test_can_create_root() {
        let mut engine = Engine::default();

        engine.create_root("kjuulh").unwrap();

        assert_eq!(
            r#"{
  "kjuulh": {
    "type": "user"
  }
}"#,
            engine.to_string()
        );
    }

    #[test]
    fn test_can_create_section() {
        let mut engine = Engine::default();

        engine.create_root("kjuulh").unwrap();
        engine
            .create(
                "kjuulh",
                &["some-section"],
                crate::log::GraphItem::Section(BTreeMap::default()),
            )
            .unwrap();

        assert_eq!(
            r#"{
  "kjuulh": {
    "type": "user",
    "some-section": {
      "type": "section"
    }
  }
}"#,
            engine.to_string()
        );
    }

    #[test]
    fn test_can_create_subsection() {
        let mut engine = Engine::default();

        engine.create_root("kjuulh").unwrap();
        engine
            .create(
                "kjuulh",
                &["some-section"],
                crate::log::GraphItem::Section(BTreeMap::default()),
            )
            .unwrap();

        engine
            .create(
                "kjuulh",
                &["some-section", "some-sub-section"],
                crate::log::GraphItem::Section(BTreeMap::default()),
            )
            .unwrap();

        assert_eq!(
            r#"{
  "kjuulh": {
    "type": "user",
    "some-section": {
      "type": "section",
      "some-sub-section": {
        "type": "section"
      }
    }
  }
}"#,
            engine.to_string()
        );
    }

    #[test]
    fn test_can_create_item() {
        let mut engine = Engine::default();

        engine.create_root("kjuulh").unwrap();
        engine
            .create(
                "kjuulh",
                &["some-item"],
                GraphItem::Item {
                    title: "some-title".to_string(),
                    description: "some-description".to_string(),
                    state: ItemState::NotDone,
                },
            )
            .unwrap();

        assert_eq!(
            r#"{
  "kjuulh": {
    "type": "user",
    "some-item": {
      "type": "item",
      "title": "some-title",
      "description": "some-description",
      "state": "not-done"
    }
  }
}"#,
            engine.to_string()
        );
    }

    #[test]
    fn test_can_create_nested_item() {
        let mut engine = Engine::default();

        engine.create_root("kjuulh").unwrap();
        engine
            .create(
                "kjuulh",
                &["some-section"],
                GraphItem::Section(BTreeMap::default()),
            )
            .unwrap();
        engine
            .create(
                "kjuulh",
                &["some-section", "some-item"],
                GraphItem::Item {
                    title: "some-title".to_string(),
                    description: "some-description".to_string(),
                    state: ItemState::NotDone,
                },
            )
            .unwrap();

        assert_eq!(
            r#"{
  "kjuulh": {
    "type": "user",
    "some-section": {
      "type": "section",
      "some-item": {
        "type": "item",
        "title": "some-title",
        "description": "some-description",
        "state": "not-done"
      }
    }
  }
}"#,
            engine.to_string()
        );
    }

    #[test]
    fn test_can_create_deeply_nested_item() {
        let mut engine = Engine::default();

        engine.create_root("kjuulh").unwrap();
        engine
            .create(
                "kjuulh",
                &["some-section"],
                GraphItem::Section(BTreeMap::default()),
            )
            .unwrap();
        engine
            .create(
                "kjuulh",
                &["some-section", "some-sub-section"],
                GraphItem::Section(BTreeMap::default()),
            )
            .unwrap();
        engine
            .create(
                "kjuulh",
                &["some-section", "some-sub-section", "sub-sub-section"],
                GraphItem::Section(BTreeMap::default()),
            )
            .unwrap();
        engine
            .create(
                "kjuulh",
                &[
                    "some-section",
                    "some-sub-section",
                    "sub-sub-section",
                    "some-item",
                ],
                GraphItem::Item {
                    title: "some-title".to_string(),
                    description: "some-description".to_string(),
                    state: ItemState::NotDone,
                },
            )
            .unwrap();

        assert_eq!(
            r#"{
  "kjuulh": {
    "type": "user",
    "some-section": {
      "type": "section",
      "some-sub-section": {
        "type": "section",
        "sub-sub-section": {
          "type": "section",
          "some-item": {
            "type": "item",
            "title": "some-title",
            "description": "some-description",
            "state": "not-done"
          }
        }
      }
    }
  }
}"#,
            engine.to_string()
        );
    }

    #[test]
    fn test_can_get_user() {
        let engine = get_complex_graph();

        let res = engine.get("kjuulh", &[]).unwrap();

        let actual = serde_json::to_string_pretty(res).unwrap();

        let expected = r#"{
  "type": "user",
  "some-section": {
    "type": "section",
    "some-sub-section": {
      "type": "section",
      "sub-sub-section": {
        "type": "section",
        "some-item": {
          "type": "item",
          "title": "some-title",
          "description": "some-description",
          "state": "not-done"
        }
      }
    }
  }
}"#;

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_can_get_first_section() {
        let engine = get_complex_graph();

        let res = engine.get("kjuulh", &["some-section"]).unwrap();

        let actual = serde_json::to_string_pretty(res).unwrap();

        let expected = r#"{
  "type": "section",
  "some-sub-section": {
    "type": "section",
    "sub-sub-section": {
      "type": "section",
      "some-item": {
        "type": "item",
        "title": "some-title",
        "description": "some-description",
        "state": "not-done"
      }
    }
  }
}"#;

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_can_get_middle_section() {
        let engine = get_complex_graph();

        let res = engine
            .get("kjuulh", &["some-section", "some-sub-section"])
            .unwrap();

        let actual = serde_json::to_string_pretty(res).unwrap();

        let expected = r#"{
  "type": "section",
  "sub-sub-section": {
    "type": "section",
    "some-item": {
      "type": "item",
      "title": "some-title",
      "description": "some-description",
      "state": "not-done"
    }
  }
}"#;

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_can_get_last_section() {
        let engine = get_complex_graph();

        let res = engine
            .get(
                "kjuulh",
                &["some-section", "some-sub-section", "sub-sub-section"],
            )
            .unwrap();

        let actual = serde_json::to_string_pretty(res).unwrap();

        let expected = r#"{
  "type": "section",
  "some-item": {
    "type": "item",
    "title": "some-title",
    "description": "some-description",
    "state": "not-done"
  }
}"#;

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_can_get_item() {
        let engine = get_complex_graph();

        let res = engine
            .get(
                "kjuulh",
                &[
                    "some-section",
                    "some-sub-section",
                    "sub-sub-section",
                    "some-item",
                ],
            )
            .unwrap();

        let actual = serde_json::to_string_pretty(res).unwrap();

        let expected = r#"{
  "type": "item",
  "title": "some-title",
  "description": "some-description",
  "state": "not-done"
}"#;

        assert_eq!(expected, actual)
    }

    #[test]
    fn test_can_move_item() {
        let mut engine = get_complex_graph();

        engine
            .section_move(
                "kjuulh",
                &[
                    "some-section",
                    "some-sub-section",
                    "sub-sub-section",
                    "some-item",
                ],
                &["some-section"],
            )
            .unwrap();

        let expected = r#"{
  "kjuulh": {
    "type": "user",
    "some-section": {
      "type": "section",
      "some-item": {
        "type": "item",
        "title": "some-title",
        "description": "some-description",
        "state": "not-done"
      },
      "some-sub-section": {
        "type": "section",
        "sub-sub-section": {
          "type": "section"
        }
      }
    }
  }
}"#;

        assert_eq!(expected, engine.to_string())
    }

    #[test]
    fn test_can_move_section() {
        let mut engine = get_complex_graph();

        engine
            .section_move(
                "kjuulh",
                &["some-section", "some-sub-section", "sub-sub-section"],
                &["some-section"],
            )
            .unwrap();

        let expected = r#"{
  "kjuulh": {
    "type": "user",
    "some-section": {
      "type": "section",
      "some-sub-section": {
        "type": "section"
      },
      "sub-sub-section": {
        "type": "section",
        "some-item": {
          "type": "item",
          "title": "some-title",
          "description": "some-description",
          "state": "not-done"
        }
      }
    }
  }
}"#;

        assert_eq!(expected, engine.to_string())
    }

    #[test]
    fn test_can_delete_section() {
        let mut engine = get_complex_graph();

        let res = engine.delete("kjuulh", &["some-section", "some-sub-section"]);

        assert!(res.is_ok());
        assert_eq!(
            r#"{
  "kjuulh": {
    "type": "user",
    "some-section": {
      "type": "section"
    }
  }
}"#,
            &engine.to_string()
        );
    }

    #[test]
    fn test_can_delete_item() {
        let mut engine = get_complex_graph();

        let res = engine.delete(
            "kjuulh",
            &[
                "some-section",
                "some-sub-section",
                "sub-sub-section",
                "some-item",
            ],
        );

        assert!(res.is_ok());
        assert_eq!(
            r#"{
  "kjuulh": {
    "type": "user",
    "some-section": {
      "type": "section",
      "some-sub-section": {
        "type": "section",
        "sub-sub-section": {
          "type": "section"
        }
      }
    }
  }
}"#,
            &engine.to_string()
        );
    }

    fn get_complex_graph() -> Engine {
        let mut engine = Engine::default();

        engine.create_root("kjuulh").unwrap();
        engine
            .create(
                "kjuulh",
                &["some-section"],
                GraphItem::Section(BTreeMap::default()),
            )
            .unwrap();
        engine
            .create(
                "kjuulh",
                &["some-section", "some-sub-section"],
                GraphItem::Section(BTreeMap::default()),
            )
            .unwrap();
        engine
            .create(
                "kjuulh",
                &["some-section", "some-sub-section", "sub-sub-section"],
                GraphItem::Section(BTreeMap::default()),
            )
            .unwrap();
        engine
            .create(
                "kjuulh",
                &[
                    "some-section",
                    "some-sub-section",
                    "sub-sub-section",
                    "some-item",
                ],
                GraphItem::Item {
                    title: "some-title".to_string(),
                    description: "some-description".to_string(),
                    state: ItemState::NotDone,
                },
            )
            .unwrap();

        engine
    }
}
