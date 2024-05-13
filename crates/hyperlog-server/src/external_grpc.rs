use hyperlog_protos::hyperlog::{
    graph_server::{Graph, GraphServer},
    *,
};
use std::{collections::HashMap, net::SocketAddr};
use tonic::{transport, Response};

use crate::{
    commands::{Command, Commander, CommanderExt},
    querier::{Querier, QuerierExt},
    state::SharedState,
};

#[allow(dead_code)]
pub struct Server {
    querier: Querier,
    commander: Commander,
}

impl Server {
    pub fn new(querier: Querier, commander: Commander) -> Self {
        Self { querier, commander }
    }
}

#[tonic::async_trait]
impl Graph for Server {
    async fn create_item(
        &self,
        request: tonic::Request<CreateItemRequest>,
    ) -> std::result::Result<tonic::Response<CreateItemResponse>, tonic::Status> {
        let req = request.into_inner();
        tracing::trace!("create item: req({:?})", req);

        if req.root.is_empty() {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "root cannot be empty".to_string(),
            ));
        }

        if req.path.is_empty() {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "path cannot be empty".to_string(),
            ));
        }

        if req
            .path
            .iter()
            .filter(|item| item.is_empty())
            .collect::<Vec<_>>()
            .first()
            .is_some()
        {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "path cannot contain empty paths".to_string(),
            ));
        }

        if req
            .path
            .iter()
            .filter(|item| item.contains("."))
            .collect::<Vec<_>>()
            .first()
            .is_some()
        {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "path cannot contain `.`".to_string(),
            ));
        }

        let item = match req.item {
            Some(i) => i,
            None => {
                return Err(tonic::Status::new(
                    tonic::Code::InvalidArgument,
                    "item cannot contain empty or null".to_string(),
                ));
            }
        };

        self.commander
            .execute(Command::CreateItem {
                root: req.root,
                path: req.path,
                title: item.title,
                description: item.description,
                state: match item.item_state {
                    Some(item_graph_item::ItemState::Done(_)) => {
                        hyperlog_core::log::ItemState::Done
                    }
                    Some(item_graph_item::ItemState::NotDone(_)) => {
                        hyperlog_core::log::ItemState::NotDone
                    }
                    None => hyperlog_core::log::ItemState::default(),
                },
            })
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(CreateItemResponse {}))
    }

    async fn create_root(
        &self,
        request: tonic::Request<CreateRootRequest>,
    ) -> std::result::Result<tonic::Response<CreateRootResponse>, tonic::Status> {
        let req = request.into_inner();
        tracing::trace!("create root: req({:?})", req);

        if req.root.is_empty() {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "root cannot be empty".to_string(),
            ));
        }

        self.commander
            .execute(Command::CreateRoot { root: req.root })
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(CreateRootResponse {}))
    }

    async fn create_section(
        &self,
        request: tonic::Request<CreateSectionRequest>,
    ) -> std::result::Result<tonic::Response<CreateSectionResponse>, tonic::Status> {
        let req = request.into_inner();
        tracing::trace!("create section: req({:?})", req);

        if req.root.is_empty() {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "root cannot be empty".to_string(),
            ));
        }

        if req.path.is_empty() {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "path cannot be empty".to_string(),
            ));
        }

        if req
            .path
            .iter()
            .filter(|item| item.is_empty())
            .collect::<Vec<_>>()
            .first()
            .is_some()
        {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "path cannot contain empty paths".to_string(),
            ));
        }

        if req
            .path
            .iter()
            .filter(|item| item.contains("."))
            .collect::<Vec<_>>()
            .first()
            .is_some()
        {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "path cannot contain `.`".to_string(),
            ));
        }

        self.commander
            .execute(Command::CreateSection {
                root: req.root,
                path: req.path,
            })
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(CreateSectionResponse {}))
    }

    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> std::result::Result<tonic::Response<GetReply>, tonic::Status> {
        let msg = request.get_ref();

        tracing::trace!("get: req({:?})", msg);

        Ok(Response::new(GetReply {
            item: Some(GraphItem {
                path: "kjuulh".into(),
                contents: Some(graph_item::Contents::User(UserGraphItem {
                    items: HashMap::from([(
                        "some".to_string(),
                        GraphItem {
                            path: "some".into(),
                            contents: Some(graph_item::Contents::Item(ItemGraphItem {
                                title: "some-title".into(),
                                description: "some-description".into(),
                                item_state: Some(item_graph_item::ItemState::NotDone(
                                    ItemStateNotDone {},
                                )),
                            })),
                        },
                    )]),
                })),
            }),
        }))
    }

    async fn get_available_roots(
        &self,
        request: tonic::Request<GetAvailableRootsRequest>,
    ) -> std::result::Result<tonic::Response<GetAvailableRootsResponse>, tonic::Status> {
        let req = request.into_inner();
        tracing::trace!("get available roots: req({:?})", req);

        Ok(Response::new(GetAvailableRootsResponse {
            roots: vec!["kjuulh".into()],
        }))
    }
}

// TODO: create more defined protobuf categories for errors
fn to_tonic_err(err: anyhow::Error) -> tonic::Status {
    tonic::Status::new(tonic::Code::Unknown, err.to_string())
}

pub trait ServerExt {
    fn grpc_server(&self) -> Server;
}

impl ServerExt for SharedState {
    fn grpc_server(&self) -> Server {
        Server::new(self.querier(), self.commander())
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
