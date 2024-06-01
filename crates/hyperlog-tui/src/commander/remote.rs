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
                let channel = self.channel.clone();

                let mut client = GraphClient::new(channel);

                let request = tonic::Request::new(CreateRootRequest { root });
                let response = client.create_root(request).await?;
                let res = response.into_inner();
            }
            Command::CreateSection { root, path } => {
                let channel = self.channel.clone();

                let mut client = GraphClient::new(channel);

                let request = tonic::Request::new(CreateSectionRequest { root, path });
                let response = client.create_section(request).await?;
                let res = response.into_inner();
            }
            Command::CreateItem {
                root,
                path,
                title,
                description,
                state,
            } => {
                let channel = self.channel.clone();

                let mut client = GraphClient::new(channel);

                let request = tonic::Request::new(CreateItemRequest {
                    root,
                    path,
                    item: Some(ItemGraphItem {
                        title,
                        description,
                        item_state: Some(match state {
                            hyperlog_core::log::ItemState::NotDone => {
                                item_graph_item::ItemState::NotDone(ItemStateNotDone {})
                            }
                            hyperlog_core::log::ItemState::Done => {
                                item_graph_item::ItemState::Done(ItemStateDone {})
                            }
                        }),
                    }),
                });
                let response = client.create_item(request).await?;
                let res = response.into_inner();
            }
            Command::Move { root, src, dest } => {
                todo!()
            }
            Command::ToggleItem { root, path } => {
                let channel = self.channel.clone();

                let mut client = GraphClient::new(channel);

                let request = tonic::Request::new(ToggleItemRequest { root, path });
                let response = client.toggle_item(request).await?;
                let res = response.into_inner();
            }
            Command::UpdateItem {
                root,
                path,
                title,
                description,
                state,
            } => {
                let channel = self.channel.clone();

                let mut client = GraphClient::new(channel);

                let request = tonic::Request::new(UpdateItemRequest {
                    root,
                    path,
                    item: Some(ItemGraphItem {
                        title,
                        description,
                        item_state: Some(match state {
                            hyperlog_core::log::ItemState::NotDone => {
                                item_graph_item::ItemState::NotDone(ItemStateNotDone {})
                            }
                            hyperlog_core::log::ItemState::Done => {
                                item_graph_item::ItemState::Done(ItemStateDone {})
                            }
                        }),
                    }),
                });
                let response = client.update_item(request).await?;
                let res = response.into_inner();
            }
            Command::Archive { root, path } => {
                let channel = self.channel.clone();

                let mut client = GraphClient::new(channel);

                let request = tonic::Request::new(ArchiveRequest { root, path });
                let response = client.archive(request).await?;
                let res = response.into_inner();
            }
        }

        Ok(())
    }
}
