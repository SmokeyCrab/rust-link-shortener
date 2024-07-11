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

use regex::Regex;

mod responses;
use responses::hyper_template_funcs;

use crate::db;
use crate::config;
use crate::connection;

fn valid_shortened_url(url: &str) -> bool {
    let re = Regex::new(r"^/.{5}").unwrap();
    return re.is_match(url);
}

async fn url_shorten_service(
    client: Arc<connection::ClientContext>,
    pg_client: Arc<tokio_postgres::Client>,
    host_config: Arc<config::HostConfig>,
    pg_config: Arc<config::PostgresConfig>,
    req: Request<hyper::body::Incoming>
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    println!("✅ {}:{} ➡️ {} {}", client.ip, client.port, req.method(), req.uri().path());
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            // println!("Request Received GET@\"/\"-{}",req.)
            Ok(responses::home(host_config)?)
        }
        (&Method::GET, "/favicon.ico") => { Ok(responses::favicon()?) }
        (&Method::POST, "/") => { Ok(responses::fail()?) }
        (&Method::GET , _) if valid_shortened_url(req.uri().path()) => {
            println!("HERE");
            Ok(responses::fail()?)
        }

        _ => { Ok(responses::fail()?) }
    }
}

pub fn create_service<'a>(
    client: Arc<connection::ClientContext>,
    pg_client: Arc<tokio_postgres::Client>,
    host_config: Arc<config::HostConfig>,
    pg_config: Arc<config::PostgresConfig>
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
        let pg_client_clone = Arc::clone(&pg_client);
        let host_config_clone = Arc::clone(&host_config);
        let pg_config_clone = Arc::clone(&pg_config);
        Box::pin(
            url_shorten_service(
                client_clone,
                pg_client_clone,
                host_config_clone,
                pg_config_clone,
                req
            )
        )
    }
}
