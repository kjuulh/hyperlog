use std::{collections::HashMap, net::SocketAddr};

use hyperlog_protos::hyperlog::{
    graph_server::{Graph, GraphServer},
    *,
};
use tonic::{transport, Response};

use crate::{
    querier::{Querier, QuerierExt},
    state::SharedState,
};

pub struct Server {
    querier: Querier,
}

impl Server {
    pub fn new(querier: Querier) -> Self {
        Self { querier }
    }
}

#[tonic::async_trait]
impl Graph for Server {
    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> std::result::Result<tonic::Response<GetReply>, tonic::Status> {
        let msg = request.get_ref();

        tracing::trace!("get: req({:?})", msg);

        Ok(Response::new(GetReply {
            items: vec![
                GraphItem {
                    path: "some.path".into(),
                    contents: Some(graph_item::Contents::Item(ItemGraphItem {
                        title: "some-title".into(),
                        description: "some-description".into(),
                        state: ItemState::NotDone as i32,
                    })),
                },
                GraphItem {
                    path: "some.path.section".into(),
                    contents: Some(graph_item::Contents::Section(SectionGraphItem {
                        items: HashMap::new(),
                    })),
                },
                GraphItem {
                    path: "some.path".into(),
                    contents: Some(graph_item::Contents::User(UserGraphItem {
                        items: HashMap::new(),
                    })),
                },
            ],
        }))
    }
}

pub trait ServerExt {
    fn grpc_server(&self) -> Server;
}

impl ServerExt for SharedState {
    fn grpc_server(&self) -> Server {
        Server::new(self.querier())
    }
}

pub async fn serve(state: &SharedState, host: SocketAddr) -> anyhow::Result<()> {
    tracing::info!("listening on {}", host);

    let graph_server = state.grpc_server();

    transport::Server::builder()
        .add_service(GraphServer::new(graph_server))
        .serve(host)
        .await?;

    Ok(())
}
