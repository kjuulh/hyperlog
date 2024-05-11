use hyperlog_core::log::{GraphItem, ItemState};

pub enum Command {
    CreateRoot {
        root: String,
    },
    CreateSection {
        root: String,
        path: Vec<String>,
    },
    CreateItem {
        root: String,
        path: Vec<String>,
        title: String,
        description: String,
        state: ItemState,
    },
    UpdateItem {
        root: String,
        path: Vec<String>,
        title: String,
        description: String,
        state: ItemState,
    },
    ToggleItem {
        root: String,
        path: Vec<String>,
    },
    Move {
        root: String,
        src: Vec<String>,
        dest: Vec<String>,
    },
}

pub struct Commander {}

impl Commander {
    pub fn execute(&self, cmd: Command) -> anyhow::Result<()> {
        match cmd {
            Command::CreateRoot { root } => todo!(),
            Command::CreateSection { root, path } => todo!(),
            Command::CreateItem {
                root,
                path,
                title,
                description,
                state,
            } => todo!(),
            Command::UpdateItem {
                root,
                path,
                title,
                description,
                state,
            } => todo!(),
            Command::ToggleItem { root, path } => todo!(),
            Command::Move { root, src, dest } => todo!(),
        }

        Ok(())
    }

    pub async fn create_root(&self, root: &str) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn create(&self, root: &str, path: &[&str], item: GraphItem) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn get(&self, root: &str, path: &[&str]) -> Option<GraphItem> {
        todo!()
    }

    pub async fn section_move(
        &self,
        root: &str,
        src_path: &[&str],
        dest_path: &[&str],
    ) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn delete(&self, root: &str, path: &[&str]) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn update_item(
        &self,
        root: &str,
        path: &[&str],
        item: &GraphItem,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
