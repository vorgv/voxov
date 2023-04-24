extern crate http_body_util;

use std::error::Error;
use std::net::SocketAddr;
use std::convert::Infallible;

use hyper::server::conn::http1;
use hyper::{body::Bytes, service::service_fn, Request, Response};
use tokio::net::TcpListener;
use http_body_util::Full;

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
        self.serve_static().await
        //TODO serve metadata
        //TODO serve config
    }
    /// Serve static big files
    async fn serve_static(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(self.static_addr).await?;
        loop {
            let (stream, _) = listener.accept().await?;
            tokio::task::spawn(async move {
                if let Err(err) = http1::Builder::new()
                    .serve_connection(stream, service_fn(Self::hello))
                    .await
                {
                    println!("Error serving: {:?}", err);
                }
            });
        }
        // SET file
        // GET file
    }
    async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        Ok(Response::new(Full::new(Bytes::from("Hello World!"))))
    }
}
