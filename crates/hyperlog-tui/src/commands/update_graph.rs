use itertools::Itertools;

use crate::{
    models::{GraphUpdatedEvent, Msg},
    querier::Querier,
    state::SharedState,
};

use super::IntoCommand;

pub struct UpdateGraphCommand {
    querier: Querier,
}

impl UpdateGraphCommand {
    pub fn new(querier: Querier) -> Self {
        Self { querier }
    }

    pub fn command(self, root: &str, path: &[&str]) -> super::Command {
        let root = root.to_owned();
        let path = path.iter().map(|i| i.to_string()).collect_vec();

        super::Command::new(|dispatch| {
            tokio::spawn(async move {
                let now = std::time::SystemTime::now();
                dispatch.send(Msg::GraphUpdated(GraphUpdatedEvent::Initiated));

                #[cfg(debug_assertions)]
                {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }

                match self
                    .querier
                    .get_async(&root, path)
                    .await
                    .ok_or(anyhow::anyhow!("failed to find path"))
                {
                    Ok(graph) => {
                        dispatch.send(Msg::GraphUpdated(GraphUpdatedEvent::Success(graph)))
                    }
                    Err(e) => dispatch.send(Msg::GraphUpdated(GraphUpdatedEvent::Failure(
                        format!("{e}"),
                    ))),
                }

                let elapsed = now.elapsed().expect("to be able to get time");
                tracing::trace!("UpdateGraphCommand took: {}nanos", elapsed.as_nanos());
            });

            None
        })
    }
}

pub trait UpdateGraphCommandExt {
    fn update_graph_command(&self) -> UpdateGraphCommand;
}

impl UpdateGraphCommandExt for SharedState {
    fn update_graph_command(&self) -> UpdateGraphCommand {
        UpdateGraphCommand::new(self.querier.clone())
    }
}
