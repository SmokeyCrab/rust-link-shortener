use tokio_postgres::{ NoTls, Error, Client, Connection, Socket };
use tokio_postgres::tls::{ NoTlsStream };
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;

use hyper::body::Frame;
use hyper::{ body::Body, Method, Request, Response, StatusCode };
use http_body_util::{ combinators::BoxBody, BodyExt, Empty, Full };

use std::future::Future;
use std::boxed::Box;
use std::pin::Pin;
use std::sync::Arc;

use sprintf::sprintf;

mod hyper_template_funcs;

use crate::db;
use crate::config;

async fn post_service(
    client: Arc<tokio_postgres::Client>,
    host_config: Arc<config::HostConfig>,
    req: Request<hyper::body::Incoming>
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") =>
            Ok(
                Response::new(
                    hyper_template_funcs::full(
                        sprintf!(
                            "Welcome to the URL Shortener, POST a URL to \"%s/\"",
                            host_config.host_url
                        ).unwrap()
                    )
                )
            ),

        //TODO Write posts into db
        //TODO Read GET from db

        _ => {
            let mut not_found = Response::new(hyper_template_funcs::empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

pub fn create_service<'a>(
    client: Arc<tokio_postgres::Client>,
    host_config: Arc<config::HostConfig>
) -> impl Fn(
    Request<hyper::body::Incoming>
) -> Pin<
    Box<
        dyn Future<Output = Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error>> +
            Send +
            'a
    >
> {
    move |req: Request<hyper::body::Incoming>| {
        let client_clone = Arc::clone(&client);
        let host_config_clone = Arc::clone(&host_config);
        Box::pin(post_service(client_clone, host_config_clone, req))
    }
}
