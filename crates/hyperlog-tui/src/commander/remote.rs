use hyperlog_protos::hyperlog::{graph_client::GraphClient, *};
use tonic::transport::Channel;

use super::Command;

#[allow(dead_code, unused_variables)]
#[derive(Clone)]
pub struct Commander {
    channel: Channel,
}

#[allow(dead_code, unused_variables)]
impl Commander {
    pub fn new(channel: Channel) -> anyhow::Result<Self> {
        Ok(Self { channel })
    }

    pub async fn execute(&self, cmd: Command) -> anyhow::Result<()> {
        tracing::debug!("executing event: {}", serde_json::to_string(&cmd)?);

        match cmd.clone() {
            Command::CreateRoot { root } => {
                todo!()
                //self.engine.create_root(&root)?;
            }
            Command::CreateSection { root, path } => {
                let channel = self.channel.clone();

                let mut client = GraphClient::new(channel);

                let request = tonic::Request::new(CreateSectionRequest {});
                let response = client.create_section(request).await?;
                let res = response.into_inner();

                // self.engine.create(
                //     &root,
                //     &path.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                //     GraphItem::Section(BTreeMap::default()),
                // )?;
            }
            Command::CreateItem {
                root,
                path,
                title,
                description,
                state,
            } => {
                todo!()
                // self.engine.create(
                //             &root,
                //             &path.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                //             GraphItem::Item {
                //                 title,
                //                 description,
                //                 state,
                //             },
                //         )?
            }
            Command::Move { root, src, dest } => {
                todo!()
                // self.engine.section_move(
                //             &root,
                //             &src.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                //             &dest.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                //         )?
            }
            Command::ToggleItem { root, path } => {
                todo!()
                // self
                //             .engine
                //             .toggle_item(&root, &path.iter().map(|p| p.as_str()).collect::<Vec<_>>())?
            }
            Command::UpdateItem {
                root,
                path,
                title,
                description,
                state,
            } => {
                todo!()
                // self.engine.update_item(
                //             &root,
                //             &path.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
                //             GraphItem::Item {
                //                 title,
                //                 description,
                //                 state,
                //             },
                //         )?
            }
        }

        // self.storage.store(&self.engine)?;
        // self.events.enque_command(cmd)?;

        Ok(())
    }
}
