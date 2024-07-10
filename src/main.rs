use std::net::{ SocketAddr, IpAddr };
use std::str::FromStr;
use std::sync::Arc;

// Services

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

// Routing

mod db;
mod services;
mod config;
mod connection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting");

    println!("Reading from config");
    let (pg_config, host_config) = config::get_config()?;
    println!("Complete!");

    println!("Connecting to DB");

    let (pg_client, pg_connection) = db::start_connection(
        &pg_config.postgres_user,
        &pg_config.postgres_ip,
        &pg_config.postgres_password,
        &pg_config.postgres_database_name,
        &pg_config.postgres_port
    ).await?;

    println!("Complete!");

    tokio::spawn(async move {
        if let Err(e) = pg_connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let rows = pg_client.query("SELECT * FROM tester", &[]).await?;

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
    let pg_client_pointer = Arc::new(pg_client);
    let host_config_pointer = Arc::new(host_config);
    let pg_config_pointer = Arc::new(pg_config);

    // let main_service = Arc::new(services::create_service(pg_client_pointer, host_config_pointer, pg_config_pointer));

    loop {
        let (stream, addr) = listener.accept().await?;

        let client_connection = Arc::new(connection::ClientContext::new(addr.ip(), addr.port()));
        let thread_client_connection = Arc::clone(&client_connection);
        let pg_client_clone = Arc::clone(&pg_client_pointer);
        let host_config_clone = Arc::clone(&host_config_pointer);
        let pg_config_clone = Arc::clone(&pg_config_pointer);

        println!("⚠️ New Client: {addr:?}");

        let io = TokioIo::new(stream);

        // let main_service_clone = Arc::clone(&main_service);

        let main_service = Arc::new(
            services::create_service(
                client_connection,
                pg_client_clone,
                host_config_clone,
                pg_config_clone
            )
        );

        tokio::task::spawn(async move {
            let builder = http1::Builder::new();
            if
                let Err(err) = builder.serve_connection(
                    io,
                    service_fn(move |req| main_service(req))
                ).await
            {
                eprintln!("Error serving connection {:?}", err);
            }
            println!(
                "⛔ {}:{} disconnected",
                thread_client_connection.ip,
                thread_client_connection.port
            );
        });
    }
}
