use std::{net::SocketAddr, sync::Arc};

use crate::state::{SharedState, State};

mod external_http;
mod internal_http;
mod external_grpc {
    use std::net::SocketAddr;

    use hyperlog_protos::hyperlog::{
        graph_server::{Graph, GraphServer},
        HelloReply, HelloRequest,
    };
    use tonic::{transport, Response};

    use crate::state::SharedState;

    #[derive(Default)]
    struct Server {}

    #[tonic::async_trait]
    impl Graph for Server {
        async fn say_hello(
            &self,
            request: tonic::Request<HelloRequest>,
        ) -> std::result::Result<tonic::Response<HelloReply>, tonic::Status> {
            tracing::info!("received hello request");

            Ok(Response::new(HelloReply {
                message: "hello".into(),
            }))
        }
    }

    pub async fn serve(state: &SharedState, host: SocketAddr) -> anyhow::Result<()> {
        tracing::info!("listening on {}", host);

        let graph_server = Server::default();

        transport::Server::builder()
            .add_service(GraphServer::new(graph_server))
            .serve(host)
            .await?;

        Ok(())
    }
}
mod state;

#[derive(Clone)]
pub struct ServeOptions {
    pub external_http: SocketAddr,
    pub internal_http: SocketAddr,

    pub external_grpc: SocketAddr,
}

pub async fn serve(opts: ServeOptions) -> anyhow::Result<()> {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.unwrap();
        tracing::info!("kill signal received, shutting down");
    };
    tracing::debug!("setting up dependencies");
    let state = SharedState(Arc::new(State::new().await?));

    tracing::debug!("serve starting");
    tokio::select!(
        res = external_http::serve(&state, &opts.external_http) => {
            res?
        },
        res = internal_http::serve(&state, &opts.internal_http) => {
            res?
        },
        res = external_grpc::serve(&state, opts.external_grpc) => {
            res?
        }
        () = ctrl_c => {}
    );
    tracing::debug!("serve finalized");

    Ok(())
}
