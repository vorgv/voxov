//! All have http endpoint.
//! Only GeneCall implements GraphQL.

use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::server::conn::http1;
use hyper::{body::Bytes, service::service_fn, Method, Request, Response, StatusCode};
use tokio::net::TcpListener;

use crate::auth::Auth;
use crate::config::Config;
use crate::message::Query;

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
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(stream, service_fn(move |req| handle_http(req, self.auth)))
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
) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
    match *req.method() {
        // Ping server
        Method::GET => Ok(Response::new(full("PONG"))),
        // Everything has side effect, so this is POST-only.
        Method::POST => match Query::try_from(&req) {
            Ok(q) => Ok(auth.handle(&q).await.to_response()),
            Err(_) => Ok(bad_request()),
        },
        _ => Ok(not_found()),
    }
}

// Utility functions to make Empty and Full bodies.
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

// Empty bodies with status code.
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

pub fn bad_request() -> Response<BoxBody<Bytes, Infallible>> {
    empty_with_code(StatusCode::BAD_REQUEST)
}
