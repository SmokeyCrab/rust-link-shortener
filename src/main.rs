use std::ptr;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

// Services
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio_postgres::{ NoTls, Error, Client, Connection, Socket };
use tokio_postgres::tls::{ NoTlsStream };

// Routing
use hyper::body::Frame;
use hyper::{ body::Body, Method, Request, Response, StatusCode };
use http_body_util::{ combinators::BoxBody, BodyExt, Empty, Full };
mod db;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting");
    let (big_client, big_connection) = db::start_connection(
        "HIDDEN",
        "HIDDEN",
        "HIDDEN",
        "Hello",
        "5432"
    ).await?;
    println!("Connected to DB");
    tokio::spawn(async move {
        if let Err(e) = big_connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let rows = big_client.query("SELECT * FROM tester", &[]).await?;

    let value: &str = rows[0].get(0);
    println!("{}", value);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;

    // Cannot be sent between threads safely, 
    // therefore cloning for each thread is necessary
    // for postgres client and service handler
    let big_client_pointer = Arc::new(big_client);
    let main_service = Arc::new(services::get_service(big_client_pointer));

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        let main_service_clone = Arc::clone(&main_service);

        tokio::task::spawn(async move {
            if
                let Err(err) = http1::Builder::new().serve_connection(
                    io,
                    service_fn(move |req| main_service_clone(req))
                ).await
            {
                eprintln!("Error serving connection {:?}", err);
            }
        });
    }
}
