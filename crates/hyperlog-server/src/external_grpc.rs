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

        let res = self
            .querier
            .get(&msg.root, msg.paths.clone())
            .await
            .map_err(to_tonic_err)?;

        match res {
            Some(item) => Ok(Response::new(GetReply {
                item: Some(to_native(&item).map_err(to_tonic_err)?),
            })),
            None => {
                return Err(tonic::Status::new(
                    tonic::Code::NotFound,
                    "failed to find any valid roots",
                ))
            }
        }
    }

    async fn get_available_roots(
        &self,
        request: tonic::Request<GetAvailableRootsRequest>,
    ) -> std::result::Result<tonic::Response<GetAvailableRootsResponse>, tonic::Status> {
        let req = request.into_inner();
        tracing::trace!("get available roots: req({:?})", req);

        let roots = match self
            .querier
            .get_available_roots()
            .await
            .map_err(to_tonic_err)?
        {
            Some(roots) => roots,
            None => {
                return Err(tonic::Status::new(
                    tonic::Code::NotFound,
                    "failed to find any valid roots",
                ))
            }
        };

        Ok(Response::new(GetAvailableRootsResponse { roots }))
    }

    async fn update_item(
        &self,
        request: tonic::Request<UpdateItemRequest>,
    ) -> std::result::Result<tonic::Response<UpdateItemResponse>, tonic::Status> {
        let req = request.into_inner();
        tracing::trace!("update item: req({:?})", req);

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
            .execute(Command::UpdateItem {
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

        Ok(Response::new(UpdateItemResponse {}))
    }
}

fn to_native(from: &hyperlog_core::log::GraphItem) -> anyhow::Result<GraphItem> {
    match from {
        hyperlog_core::log::GraphItem::User(section)
        | hyperlog_core::log::GraphItem::Section(section) => {
            let mut root = HashMap::new();
            for (key, value) in section.iter() {
                root.insert(key.to_string(), to_native(value)?);
            }
            match from {
                hyperlog_core::log::GraphItem::User(_) => Ok(GraphItem {
                    contents: Some(graph_item::Contents::User(UserGraphItem { items: root })),
                }),
                hyperlog_core::log::GraphItem::Section(_) => Ok(GraphItem {
                    contents: Some(graph_item::Contents::Section(SectionGraphItem {
                        items: root,
                    })),
                }),
                _ => {
                    todo!()
                }
            }
        }
        hyperlog_core::log::GraphItem::Item {
            title,
            description,
            state,
        } => Ok(GraphItem {
            contents: Some(graph_item::Contents::Item(ItemGraphItem {
                title: title.to_owned(),
                description: description.to_owned(),
                item_state: Some(match state {
                    hyperlog_core::log::ItemState::NotDone => {
                        item_graph_item::ItemState::NotDone(ItemStateNotDone {})
                    }
                    hyperlog_core::log::ItemState::Done => {
                        item_graph_item::ItemState::Done(ItemStateDone {})
                    }
                }),
            })),
        }),
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
