use std::{ops::Deref, sync::Arc};

use crate::core_state::State;

#[derive(Clone)]
pub struct SharedState {
    state: Arc<State>,
}

impl Deref for SharedState {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl From<State> for SharedState {
    fn from(value: State) -> Self {
        Self {
            state: Arc::new(value),
        }
    }
}
