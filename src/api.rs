//! All have http endpoint.
//! Only GeneCall implements GraphQL.

use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;

use axum::routing::{get, post};
use axum::Router;

use http_body_util::{BodyExt, Empty, Full};
use hyper::{body::Bytes, Method, Request, Response, StatusCode};
use tokio::net::TcpListener;

use crate::auth::Auth;
use crate::body::ResponseBody as RB;
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
    pub async fn serve(&'static self) {
        self.serve_http().await;
        //TODO tokio::spawn serve_graphql.
    }

    /// Serve plain http endpoint.
    async fn serve_http(&'static self) {
        let listener = TcpListener::bind(self.http_addr).await.unwrap();
        let app = Router::new()
            .route("/", get(|| async { "PONG" }))
            .route("/", post();
        axum::serve(listener, app).await.unwrap();
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
            Ok(q) => Ok(auth
                .handle(q)
                .await
                .unwrap_or_else(|error| crate::message::Reply::Error { error })
                .to_response()),
            Err(_) => Ok(bad_request()),
        },
        _ => Ok(not_found()),
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

// Empty bodies with status code.
fn empty_with_code(status_code: StatusCode) -> Response<RB> {
    let mut response = Response::new(empty());
    *response.status_mut() = status_code;
    response
}

fn not_found() -> Response<RB> {
    empty_with_code(StatusCode::NOT_FOUND)
}

pub fn not_implemented() -> Response<RB> {
    empty_with_code(StatusCode::NOT_IMPLEMENTED)
}

pub fn bad_request() -> Response<RB> {
    empty_with_code(StatusCode::BAD_REQUEST)
}
