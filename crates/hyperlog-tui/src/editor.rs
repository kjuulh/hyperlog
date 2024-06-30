use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::{anyhow, Context};
use hyperlog_core::log::{GraphItem, ItemState};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::project_dirs::get_project_dir;

pub struct EditorSession<'a> {
    item: &'a GraphItem,
}

struct EditorFile {
    title: String,
    metadata: Metadata,
    body: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Metadata {
    state: ItemState,
}

impl EditorFile {
    pub fn serialize(&self) -> anyhow::Result<String> {
        let metadata =
            toml::to_string_pretty(&self.metadata).context("failed to serialize metadata")?;

        let frontmatter = format!("+++\n{}+++\n", metadata);

        Ok(format!(
            "{}\n# {}\n\n{}",
            frontmatter, self.title, self.body
        ))
    }
}

impl TryFrom<&GraphItem> for EditorFile {
    type Error = anyhow::Error;

    fn try_from(value: &GraphItem) -> Result<Self, Self::Error> {
        if let GraphItem::Item {
            title,
            description,
            state,
        } = value.clone()
        {
            Ok(Self {
                title,
                metadata: Metadata { state },
                body: description,
            })
        } else {
            anyhow::bail!("can only generate a file based on items")
        }
    }
}

impl TryFrom<&str> for EditorFile {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.to_string();

        let frontmatter_parts = value.split("+++").filter(|p| !p.is_empty()).collect_vec();
        let frontmatter_content = frontmatter_parts
            .first()
            .ok_or(anyhow::anyhow!("no front matter parts were found"))?;

        tracing::trace!("parsing frontmatter content: {}", frontmatter_content);
        let metadata: Metadata = toml::from_str(frontmatter_content)?;

        let line_parts = value.split("\n");

        let title = line_parts
            .clone()
            .find(|p| p.starts_with("# "))
            .map(|t| t.trim_start_matches("# "))
            .ok_or(anyhow!("an editor file requires a title with heading 1"))?;
        let body = line_parts
            .skip_while(|p| !p.starts_with("# "))
            .skip(1)
            .skip_while(|p| p.is_empty())
            .collect_vec()
            .join("\n");

        Ok(Self {
            title: title.to_string(),
            metadata,
            body,
        })
    }
}

impl From<EditorFile> for GraphItem {
    fn from(value: EditorFile) -> Self {
        Self::Item {
            title: value.title,
            description: value.body,
            state: value.metadata.state,
        }
    }
}

struct SessionFile {
    path: PathBuf,
    loaded: SystemTime,
}

impl SessionFile {
    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn is_changed(&self) -> anyhow::Result<bool> {
        let modified = self.path.metadata()?.modified()?;

        Ok(self.loaded < modified)
    }
}

impl Drop for SessionFile {
    fn drop(&mut self) {
        if self.path.exists() {
            tracing::debug!("cleaning up file: {}", self.path.display());

            if let Err(e) = std::fs::remove_file(&self.path) {
                tracing::error!(
                    "failed to cleanup file: {}, error: {}",
                    self.path.display(),
                    e
                );
            }
        }
    }
}

impl<'a> EditorSession<'a> {
    pub fn new(item: &'a GraphItem) -> Self {
        Self { item }
    }

    fn get_file_path(&mut self) -> anyhow::Result<PathBuf> {
        let name = self
            .item
            .get_digest()
            .ok_or(anyhow::anyhow!("item doesn't have a title"))?;

        let file_path = get_project_dir()
            .data_dir()
            .join("edit")
            .join(format!("{name}.md"));

        Ok(file_path)
    }

    fn prepare_file(&mut self) -> anyhow::Result<SessionFile> {
        let file_path = self.get_file_path()?;

        if let Some(parent) = file_path.parent() {
            tracing::debug!("creating parent dir: {}", parent.display());
            std::fs::create_dir_all(parent).context("failed to create dir for edit file")?;
        }

        let mut file =
            std::fs::File::create(&file_path).context("failed to create file for edit file")?;

        tracing::debug!("writing contents to file: {}", file_path.display());
        let editor_file = EditorFile::try_from(self.item)?;
        file.write_all(
            editor_file
                .serialize()
                .context("failed to serialize item to file")?
                .as_bytes(),
        )
        .context("failed to write to file")?;

        let modified_time = file.metadata()?.modified()?;

        Ok(SessionFile {
            path: file_path,
            loaded: modified_time,
        })
    }

    fn get_item_from_file(&self, session_file: SessionFile) -> anyhow::Result<GraphItem> {
        let mut file = std::fs::File::open(&session_file.path)?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let editor_file = EditorFile::try_from(content.as_str())?;

        Ok(editor_file.into())
    }

    pub fn execute(&mut self) -> anyhow::Result<Option<GraphItem>> {
        let editor = std::env::var("EDITOR").context("no editor was found for EDITOR env var")?;
        let session_file = self.prepare_file()?;

        tracing::debug!(
            "opening editor: {} at path: {}",
            editor,
            session_file.get_path().display()
        );
        if let Err(e) = std::process::Command::new(editor)
            .arg(session_file.get_path())
            .status()
        {
            tracing::error!("failed command with: {}", e);
            return Ok(None);
        }

        tracing::debug!(
            "returning from editor, checking file: {}",
            session_file.get_path().display()
        );
        if session_file.is_changed()? {
            tracing::debug!(
                "file: {} changed, updating item",
                session_file.get_path().display()
            );

            Ok(Some(self.get_item_from_file(session_file)?))
        } else {
            Ok(None)
        }
    }
}

trait ItemExt {
    fn get_digest(&self) -> Option<String>;
}

impl<'a> ItemExt for &'a GraphItem {
    fn get_digest(&self) -> Option<String> {
        if let GraphItem::Item { title, .. } = self {
            let digest = sha2::Sha256::digest(title.as_bytes());
            let digest_hex = hex::encode(digest);

            Some(format!(
                "{}_{}",
                title
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .take(10)
                    .collect::<String>(),
                digest_hex.chars().take(10).collect::<String>()
            ))
        } else {
            None
        }
    }
}
