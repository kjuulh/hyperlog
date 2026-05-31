use hyperlog_protos::hyperlog::{
    auth_server::AuthServer,
    graph_server::{Graph, GraphServer},
    *,
};
use std::{collections::HashMap, net::SocketAddr};
use tonic::{transport, Request, Response, Status};

use crate::{
    auth::{AuthService, AuthedUser},
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
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
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
                due: Some(item.due).filter(|s| !s.is_empty()),
                links: item
                    .links
                    .into_iter()
                    .map(|l| hyperlog_core::log::Link { title: l.title, url: l.url })
                    .collect(),
            }, user_id)
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(CreateItemResponse {}))
    }

    async fn create_root(
        &self,
        request: tonic::Request<CreateRootRequest>,
    ) -> std::result::Result<tonic::Response<CreateRootResponse>, tonic::Status> {
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
        let req = request.into_inner();
        tracing::trace!("create root: req({:?})", req);

        if req.root.is_empty() {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "root cannot be empty".to_string(),
            ));
        }

        self.commander
            .execute(Command::CreateRoot { root: req.root }, user_id)
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(CreateRootResponse {}))
    }

    async fn create_section(
        &self,
        request: tonic::Request<CreateSectionRequest>,
    ) -> std::result::Result<tonic::Response<CreateSectionResponse>, tonic::Status> {
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
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
            .execute(
                Command::CreateSection {
                    root: req.root,
                    path: req.path,
                },
                user_id,
            )
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(CreateSectionResponse {}))
    }

    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> std::result::Result<tonic::Response<GetReply>, tonic::Status> {
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
        let msg = request.get_ref();

        tracing::trace!("get: req({:?})", msg);

        let res = self
            .querier
            .get(&msg.root, msg.paths.clone(), user_id)
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
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
        let req = request.into_inner();
        tracing::trace!("get available roots: req({:?})", req);

        let roots = match self
            .querier
            .get_available_roots(user_id)
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
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
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
                due: Some(item.due).filter(|s| !s.is_empty()),
                links: item
                    .links
                    .into_iter()
                    .map(|l| hyperlog_core::log::Link { title: l.title, url: l.url })
                    .collect(),
            }, user_id)
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(UpdateItemResponse {}))
    }

    async fn toggle_item(
        &self,
        request: tonic::Request<ToggleItemRequest>,
    ) -> std::result::Result<tonic::Response<ToggleItemResponse>, tonic::Status> {
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
        let req = request.into_inner();
        tracing::trace!("toggle item: req({:?})", req);

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
            .execute(
                Command::ToggleItem {
                    root: req.root,
                    path: req.path,
                },
                user_id,
            )
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(ToggleItemResponse {}))
    }

    async fn archive(
        &self,
        request: tonic::Request<ArchiveRequest>,
    ) -> std::result::Result<tonic::Response<ArchiveResponse>, tonic::Status> {
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
        let req = request.into_inner();
        tracing::trace!("archive: req({:?})", req);

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

        self.commander
            .execute(
                Command::Archive {
                    root: req.root,
                    path: req.path,
                },
                user_id,
            )
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(ArchiveResponse {}))
    }

    async fn restore(
        &self,
        request: tonic::Request<RestoreRequest>,
    ) -> std::result::Result<tonic::Response<RestoreResponse>, tonic::Status> {
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
        let req = request.into_inner();
        tracing::trace!("restore: req({:?})", req);

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

        self.commander
            .execute(
                Command::Restore {
                    root: req.root,
                    path: req.path,
                },
                user_id,
            )
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(RestoreResponse {}))
    }

    async fn r#move(
        &self,
        request: tonic::Request<MoveRequest>,
    ) -> std::result::Result<tonic::Response<MoveResponse>, tonic::Status> {
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
        let req = request.into_inner();
        tracing::trace!("move: req({:?})", req);

        if req.root.is_empty() {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "root cannot be empty".to_string(),
            ));
        }
        if req.src.is_empty() || req.dest.is_empty() {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "src and dest cannot be empty".to_string(),
            ));
        }

        self.commander
            .execute(
                Command::Move {
                    root: req.root,
                    src: req.src,
                    dest: req.dest,
                },
                user_id,
            )
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(MoveResponse {}))
    }

    async fn reorder(
        &self,
        request: tonic::Request<ReorderRequest>,
    ) -> std::result::Result<tonic::Response<ReorderResponse>, tonic::Status> {
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
        let req = request.into_inner();
        tracing::trace!("reorder: req({:?})", req);

        if req.root.is_empty() {
            return Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "root cannot be empty".to_string(),
            ));
        }

        self.commander
            .execute(
                Command::Reorder {
                    root: req.root,
                    path: req.path,
                    order: req.order,
                },
                user_id,
            )
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(ReorderResponse {}))
    }

    async fn get_archived(
        &self,
        request: tonic::Request<GetArchivedRequest>,
    ) -> std::result::Result<tonic::Response<GetArchivedResponse>, tonic::Status> {
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
        let req = request.into_inner();
        tracing::trace!("get archived: req({:?})", req);

        let items = self
            .querier
            .get_archived(&req.root, user_id)
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(GetArchivedResponse {
            items: items
                .into_iter()
                .map(|i| ArchivedItem {
                    path: i.path,
                    item_type: i.item_type,
                    title: i.title,
                })
                .collect(),
        }))
    }

    async fn get_view(
        &self,
        request: tonic::Request<GetViewRequest>,
    ) -> std::result::Result<tonic::Response<GetViewResponse>, tonic::Status> {
        let user_id = request.extensions().get::<AuthedUser>().map(|u| u.0);
        let req = request.into_inner();
        let max_depth = if req.max_depth <= 0 { 3 } else { req.max_depth };
        let limits = if req.limits.is_empty() {
            vec![10, 5, 3]
        } else {
            req.limits
        };
        let expanded: std::collections::HashSet<String> = req.expanded.into_iter().collect();

        let root = self
            .querier
            .get_view(&req.root, user_id, req.focus, expanded, max_depth, limits)
            .await
            .map_err(to_tonic_err)?;

        Ok(Response::new(GetViewResponse {
            root: Some(to_view_node(root)),
        }))
    }
}

fn to_view_node(v: crate::services::get_view::ViewItem) -> ViewNode {
    ViewNode {
        key: v.key,
        path: v.path,
        kind: v.kind,
        title: v.title,
        description: v.description,
        done: v.done,
        child_count: v.child_count,
        truncated: v.truncated,
        children: v.children.into_iter().map(to_view_node).collect(),
        due: v.due.unwrap_or_default(),
        created_unix: v.created_unix,
        links: v
            .links
            .into_iter()
            .map(|l| Link { title: l.title, url: l.url })
            .collect(),
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
                due: String::new(),
                created_unix: 0,
                links: Vec::new(),
            })),
        }),
    }
}

// TODO: create more defined protobuf categories for errors
fn to_tonic_err(err: anyhow::Error) -> tonic::Status {
    tonic::Status::new(tonic::Code::Unknown, err.to_string())
}

/// Extract and verify the user id from an `authorization: Bearer <jwt>` header.
fn bearer_uid(secret: &[u8], req: &Request<()>) -> Option<uuid::Uuid> {
    let val = req.metadata().get("authorization")?;
    let s = val.to_str().ok()?;
    let token = s
        .strip_prefix("Bearer ")
        .or_else(|| s.strip_prefix("bearer "))?;
    AuthService::verify_access(secret, token).ok()
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
    let auth_server = AuthService::new(state.db.clone());
    let secret = auth_server.jwt_secret();

    // When set, Graph calls REQUIRE a valid Bearer token; otherwise the token is
    // injected when present but tokenless calls still pass (legacy/no-auth mode).
    let require_auth = std::env::var("HYPERLOG_REQUIRE_AUTH")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    tracing::info!("auth enforcement on Graph: {}", require_auth);

    // Auth service interceptor: inject the user if a valid token is present,
    // never reject (register/login/refresh are public; Me self-checks).
    let secret_auth = secret.clone();
    let auth_interceptor = move |mut req: Request<()>| -> Result<Request<()>, Status> {
        if let Some(uid) = bearer_uid(secret_auth.as_slice(), &req) {
            req.extensions_mut().insert(AuthedUser(uid));
        }
        Ok(req)
    };

    // Graph interceptor: inject the user; reject when enforcement is on and no
    // valid token is present.
    let secret_graph = secret.clone();
    let graph_interceptor = move |mut req: Request<()>| -> Result<Request<()>, Status> {
        match bearer_uid(secret_graph.as_slice(), &req) {
            Some(uid) => {
                req.extensions_mut().insert(AuthedUser(uid));
                Ok(req)
            }
            None if require_auth => {
                Err(Status::unauthenticated("authentication required"))
            }
            None => Ok(req),
        }
    };

    transport::Server::builder()
        .add_service(GraphServer::with_interceptor(
            graph_server,
            graph_interceptor,
        ))
        .add_service(AuthServer::with_interceptor(auth_server, auth_interceptor))
        .serve(host)
        .await?;

    Ok(())
}
