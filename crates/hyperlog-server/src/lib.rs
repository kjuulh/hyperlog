use std::{net::SocketAddr, sync::Arc};

use crate::state::{SharedState, State};

mod external_grpc;
mod external_http;
mod internal_http;

mod commands;
mod querier;

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
