use std::ptr;

use std::convert::Infallible;
use std::net::{ SocketAddr, IpAddr };
use std::str::FromStr;
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
mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting");

    println!("Reading from config");
    let (pg_config, host_config) = config::get_config()?;
    println!("Complete!");

    println!("Connecting to DB");

    let (big_client, big_connection) = db::start_connection(
        &pg_config.postgres_user,
        &pg_config.postgres_ip,
        &pg_config.postgres_password,
        &pg_config.postgres_database_name,
        &pg_config.postgres_port
    ).await?;

    println!("Complete!");

    tokio::spawn(async move {
        if let Err(e) = big_connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let rows = big_client.query("SELECT * FROM tester", &[]).await?;

    let value: &str = rows[0].get(0);
    println!("{}", value);

    println!("Binding to port {}", &host_config.host_port);
    let addr = SocketAddr::from((
        IpAddr::from_str(&host_config.host_ip)?,
        (&host_config.host_port).parse::<u16>().unwrap(),
    ));
    let listener = TcpListener::bind(addr).await?;

    println!("Complete!");

    println!("Starting service");
    // Cannot be sent between threads safely,
    // therefore cloning for each thread is necessary
    // for postgres client and service handler
    let big_client_pointer = Arc::new(big_client);
    let host_config_pointer = Arc::new(host_config);
    let main_service = Arc::new(services::create_service(big_client_pointer, host_config_pointer));

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
