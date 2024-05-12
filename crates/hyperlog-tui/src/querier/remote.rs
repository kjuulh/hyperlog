use std::collections::BTreeMap;

use hyperlog_core::log::GraphItem;
use hyperlog_protos::hyperlog::{
    graph_client::GraphClient, graph_item::Contents, GetAvailableRootsRequest, GetRequest,
};
use itertools::Itertools;
use tonic::transport::Channel;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Querier {
    channel: Channel,
}

#[allow(dead_code, unused_variables)]
impl Querier {
    pub async fn new() -> anyhow::Result<Self> {
        let channel = Channel::from_static("http://localhost:4000")
            .connect()
            .await?;

        Ok(Self { channel })
    }

    pub async fn get_available_roots(&self) -> anyhow::Result<Option<Vec<String>>> {
        let channel = self.channel.clone();

        let mut client = GraphClient::new(channel);

        let request = tonic::Request::new(GetAvailableRootsRequest {});
        let response = client.get_available_roots(request).await?;

        let roots = response.into_inner();

        if roots.roots.is_empty() {
            Ok(None)
        } else {
            Ok(Some(roots.roots))
        }
    }

    pub async fn get(
        &self,
        root: &str,
        path: impl IntoIterator<Item = impl Into<String>>,
    ) -> anyhow::Result<Option<GraphItem>> {
        let paths = path.into_iter().map(|i| i.into()).collect_vec();

        tracing::debug!(
            "quering: root:({}), path:({}), len: ({}))",
            root,
            paths.join("."),
            paths.len()
        );

        let channel = self.channel.clone();

        let mut client = GraphClient::new(channel);

        let request = tonic::Request::new(GetRequest {
            root: root.into(),
            paths,
        });

        let response = client.get(request).await?;

        let graph_item = response.into_inner();

        if let Some(item) = graph_item.item {
            let local_graph = transform_proto_to_local(&item);
            Ok(local_graph)
        } else {
            Ok(None)
        }
    }
}

fn transform_proto_to_local(input: &hyperlog_protos::hyperlog::GraphItem) -> Option<GraphItem> {
    match &input.contents {
        Some(item) => match item {
            Contents::User(user) => {
                let mut items = BTreeMap::new();

                for (key, value) in &user.items {
                    if let Some(item) = transform_proto_to_local(value) {
                        items.insert(key.clone(), item);
                    }
                }

                Some(GraphItem::User(items))
            }
            Contents::Section(section) => {
                let mut items = BTreeMap::new();

                for (key, value) in &section.items {
                    if let Some(item) = transform_proto_to_local(value) {
                        items.insert(key.clone(), item);
                    }
                }

                Some(GraphItem::Section(items))
            }
            Contents::Item(item) => Some(GraphItem::Item {
                title: item.title.clone(),
                description: item.description.clone(),
                state: match &item.item_state {
                    Some(state) => match state {
                        hyperlog_protos::hyperlog::item_graph_item::ItemState::NotDone(_) => {
                            hyperlog_core::log::ItemState::NotDone
                        }
                        hyperlog_protos::hyperlog::item_graph_item::ItemState::Done(_) => {
                            hyperlog_core::log::ItemState::Done
                        }
                    },
                    None => hyperlog_core::log::ItemState::NotDone,
                },
            }),
        },
        None => None,
    }
}
