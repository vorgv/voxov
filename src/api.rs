//! All have http endpoint.
//! Only GeneCall implements GraphQL.

use crate::auth::Auth;
use crate::body::ResponseBody as RB;
use crate::config::Config;
use crate::ir::{Query, Reply};
use http_body_util::{BodyExt, Empty, Full};
use hyper::server::conn::http1;
use hyper::{body::Bytes, service::service_fn, Method, Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
pub struct Api {
    auth: &'static Auth,
    http_addr: SocketAddr,
}

/// Server endpoints.
impl Api {
    pub fn new(config: &Config, auth: &'static Auth) -> Api {
        Api {
            auth,
            http_addr: config.http_addr,
        }
    }

    /// Open endpoints.
    pub async fn serve(&'static self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.serve_http().await
        //TODO tokio::spawn serve_graphql.
    }

    /// Serve plain http endpoint.
    async fn serve_http(&'static self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(self.http_addr).await?;
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(io, service_fn(move |req| handle_http(req, self.auth)))
                    .await
                {
                    panic!("Error serving: {:?}", err);
                }
            });
        }
    }
}

async fn handle_http(
    req: Request<hyper::body::Incoming>,
    auth: &'static Auth,
) -> Result<Response<RB>, Infallible> {
    match *req.method() {
        // Ping server
        Method::GET => Ok(Response::new(full("PONG"))),
        // Everything has side effect, so this is POST-only.
        Method::POST => match Query::try_from(req) {
            Ok(query) => Ok(auth
                .handle(query)
                .await
                .unwrap_or_else(|error| Reply::Error { error })
                .to_response()),
            Err(error) => Ok(Reply::Error { error }.to_response()),
        },
        _ => Ok(Reply::Error {
            error: crate::Error::ApiMethod,
        }
        .to_response()),
    }
}

// Utility functions to make Empty and Full bodies.

pub fn empty() -> RB {
    RB::Box(
        Empty::<Bytes>::new()
            .map_err(|never| match never {})
            .boxed(),
    )
}

pub fn full<T: Into<Bytes>>(chunk: T) -> RB {
    RB::Box(
        Full::new(chunk.into())
            .map_err(|never| match never {})
            .boxed(),
    )
}
