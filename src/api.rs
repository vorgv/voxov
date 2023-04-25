extern crate http_body_util;

use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::server::conn::http1;
use hyper::{body::Bytes, service::service_fn, Method, Request, Response, StatusCode};
use tokio::net::TcpListener;

use crate::auth::Auth;
use crate::config::Config;

pub struct Api {
    auth: Auth,
    static_addr: SocketAddr,
}

impl Api {
    pub fn new(config: &Config, auth: Auth) -> Api {
        Api {
            auth,
            static_addr: config.static_addr,
        }
    }
    /// Open end points
    pub async fn serve(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let future_static = self.serve_static();
        //TODO serve metadata
        //TODO serve config
        future_static.await
    }
    /// Serve static big files
    async fn serve_static(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(self.static_addr).await?;
        loop {
            let (stream, _) = listener.accept().await?;
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(stream, service_fn(handle_static))
                    .await
                {
                    println!("Error serving: {:?}", err);
                }
            });
        }
    }
}

async fn handle_static(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
    match req.method() {
        // Ping server
        &Method::GET => Ok(Response::new(full("PONG"))),
        // Everything has side effect, so this is POST-only.
        &Method::POST => Ok(Response::new(full("POST"))),
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

// Utility functions to make Empty and Full bodies
fn empty() -> BoxBody<Bytes, Infallible> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, Infallible> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
