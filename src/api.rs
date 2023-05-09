use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

extern crate http_body_util;
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::server::conn::http1;
use hyper::{body::Bytes, service::service_fn, Method, Request, Response, StatusCode};
use tokio::net::TcpListener;

use crate::auth::Auth;
use crate::config::Config;
use crate::database::Database;
use crate::message::Query;

pub struct Api {
    auth: Arc<Auth>,
    static_addr: SocketAddr,
    db: &'static Database,
}

impl Api {
    pub fn new(config: &Config, db: &'static Database, auth: Auth) -> Api {
        Api {
            auth: Arc::new(auth),
            static_addr: config.static_addr,
            db,
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
            let auth = Arc::clone(&self.auth);
            let (stream, _) = listener.accept().await?;
            tokio::task::spawn(async move {
                let auth = Arc::clone(&auth);
                if let Err(err) = http1::Builder::new()
                    .serve_connection(
                        stream,
                        service_fn(move |req| handle_static(req, Arc::clone(&auth))),
                    )
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
    auth: Arc<Auth>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
    match *req.method() {
        // Ping server
        Method::GET => Ok(Response::new(full("PONG"))),
        // Everything has side effect, so this is POST-only.
        Method::POST => match Query::try_from(&req) {
            Ok(q) => Ok(auth.handle(&q).await.to_response()),
            Err(_) => Ok(not_found()),
        },
        _ => Ok(not_found()),
    }
}

// Utility functions to make Empty and Full bodies
pub fn empty() -> BoxBody<Bytes, Infallible> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, Infallible> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
fn empty_with_code(status_code: StatusCode) -> Response<BoxBody<Bytes, Infallible>> {
    let mut response = Response::new(empty());
    *response.status_mut() = status_code;
    response
}
fn not_found() -> Response<BoxBody<Bytes, Infallible>> {
    empty_with_code(StatusCode::NOT_FOUND)
}
pub fn not_implemented() -> Response<BoxBody<Bytes, Infallible>> {
    empty_with_code(StatusCode::NOT_IMPLEMENTED)
}
