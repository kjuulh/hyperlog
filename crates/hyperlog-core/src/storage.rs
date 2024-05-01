use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{engine::Engine, shared_engine::SharedEngine};

pub struct LockFile(PathBuf);

impl Drop for LockFile {
    fn drop(&mut self) {
        tracing::debug!("removing lockfile");
        std::fs::remove_file(&self.0).expect("to be able to delete lockfile")
    }
}

impl From<PathBuf> for LockFile {
    fn from(value: PathBuf) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct Storage {
    base: PathBuf,
    lock_file: Arc<Mutex<Option<LockFile>>>,
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage {
    pub fn new() -> Self {
        let data_dir = dirs::data_local_dir()
            .ok_or(anyhow::anyhow!("failed to retrieve the users data dir"))
            .expect("to be able to find config");

        Self {
            base: data_dir,
            lock_file: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_base(&mut self, base: &Path) {
        self.base = base.to_path_buf();
    }

    pub fn store(&self, engine: &SharedEngine) -> anyhow::Result<()> {
        let state_path = self.state()?;

        std::fs::write(state_path, engine.to_str()?)?;

        Ok(())
    }

    pub fn load(&self) -> anyhow::Result<Engine> {
        let mut lock = self.lock_file.lock().unwrap();
        if lock.is_none() {
            let lock_file = self.state_lock_file()?;
            *lock = Some(lock_file);
        }

        let engine = match self.state_file()? {
            Some(contents) => Engine::engine_from_str(&contents)?,
            None => Engine::default(),
        };

        Ok(engine)
    }

    pub fn unload(self) -> anyhow::Result<()> {
        drop(self);
        Ok(())
    }

    pub fn clear_lock_file(self) {
        let mut lock_file = self.lock_file.lock().unwrap();

        if lock_file.is_some() {
            *lock_file = None;
        }
    }

    fn state(&self) -> anyhow::Result<PathBuf> {
        self.cache().map(|c| c.join("graph.json"))
    }

    fn state_file(&self) -> anyhow::Result<Option<String>> {
        let state_path = self.state()?;

        if !state_path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&state_path)?;

        Ok(Some(contents))
    }

    fn state_lock(&self) -> anyhow::Result<PathBuf> {
        self.cache().map(|c| c.join("graph.lock"))
    }

    fn create_lock_file(&self) -> anyhow::Result<()> {
        let lock_path = self.state_lock()?;

        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(lock_path, "hyperlog-lock")?;

        Ok(())
    }

    fn state_lock_file(&self) -> anyhow::Result<LockFile> {
        let lock_path = self.state_lock()?;

        if !lock_path.exists() {
            self.create_lock_file()?;
            return Ok(LockFile::from(lock_path));
        }

        if let Ok(modified) = lock_path.metadata()?.modified() {
            if modified.elapsed()? > Duration::from_secs(86400) {
                std::fs::remove_file(&lock_path)?;

                self.create_lock_file()?;
                return Ok(LockFile::from(lock_path));
            }
        }

        anyhow::bail!("lock file exists and is valid. Aborting");
    }

    fn cache(&self) -> anyhow::Result<PathBuf> {
        Ok(self.base.join("hyperlog"))
    }

    pub fn info(&self) -> anyhow::Result<String> {
        Ok(format!("storage:\n\tgraph: {}", self.state()?.display()))
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use similar_asserts::assert_eq;

    use crate::log::GraphItem;

    use super::*;

    #[test]
    fn can_create_state() -> anyhow::Result<()> {
        let tempdir = tempfile::tempdir()?;

        let mut storage = Storage::default();
        storage.with_base(tempdir.path());

        let engine = SharedEngine::from(storage.load()?);
        engine.create_root("can_create_state")?;

        storage.store(&engine)?;

        let graph = std::fs::read_to_string(tempdir.path().join("hyperlog").join("graph.json"))?;
        let lock = std::fs::read_to_string(tempdir.path().join("hyperlog").join("graph.lock"))?;

        assert_eq!(
            r#"{
  "can_create_state": {
    "type": "user"
  }
}"#
            .to_string(),
            graph
        );
        assert_eq!(r#"hyperlog-lock"#.to_string(), lock);

        Ok(())
    }

    #[test]
    fn lock_already_exists() -> anyhow::Result<()> {
        let tempdir = tempfile::tempdir()?;

        let mut storage = Storage::default();
        storage.with_base(tempdir.path());

        let _engine = storage.load()?;

        let mut storage_should_fail = Storage::default();
        storage_should_fail.with_base(tempdir.path());

        let engine_should_fail = storage_should_fail.load();

        assert!(engine_should_fail.is_err());
        if let Err(e) = engine_should_fail {
            assert_eq!(
                "lock file exists and is valid. Aborting".to_string(),
                e.to_string()
            );
        }

        Ok(())
    }

    #[test]
    fn lock_is_cleaned_up() -> anyhow::Result<()> {
        let tempdir = tempfile::tempdir()?;

        let mut storage = Storage::default();
        storage.with_base(tempdir.path());

        let engine = SharedEngine::from(storage.load()?);
        engine.create_root("can_create_state")?;

        storage.store(&engine)?;
        storage.unload()?;

        assert!(!tempdir.path().join("hyperlog").join("graph.lock").exists());

        Ok(())
    }

    #[test]
    fn can_load_state() -> anyhow::Result<()> {
        let tempdir = tempfile::tempdir()?;

        let mut storage = Storage::default();
        storage.with_base(tempdir.path());

        let engine = SharedEngine::from(storage.load()?);
        engine.create_root("can_create_state")?;

        storage.store(&engine)?;

        let graph = std::fs::read_to_string(tempdir.path().join("hyperlog").join("graph.json"))?;

        assert_eq!(
            r#"{
  "can_create_state": {
    "type": "user"
  }
}"#
            .to_string(),
            graph
        );

        let engine = storage.load()?;

        let res = engine.get("can_create_state", &[]);

        assert_eq!(Some(GraphItem::User(BTreeMap::default())), res.cloned());

        Ok(())
    }
}
