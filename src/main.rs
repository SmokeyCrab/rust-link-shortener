use std::ptr;

use std::convert::Infallible;
use std::net::SocketAddr;

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

// Hyper tutorial to help me figure out the library

// async fn hello(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
//     Ok(
//         Response::new(
//             Full::new(
//                 Bytes::from(
//                     std::fs::read_to_string("interface/web.html").expect("File could not be opened")
//                 )
//             )
//         )
//     )
// }

async fn echo(
    req: Request<hyper::body::Incoming>
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(full("Echo active, use /echo for POST"))),

        // Send stream back as is
        (&Method::POST, "/echo") => { Ok(Response::new(req.into_body().boxed())) }

        // Pass stream through map
        (&Method::POST, "/echo/uppercase") => {
            let frame_stream = req.into_body().map_frame(|frame| {
                let frame = if let Ok(data) = frame.into_data() {
                    data.iter()
                        .map(|byte| byte.to_ascii_uppercase())
                        .collect::<Bytes>()
                } else {
                    Bytes::new()
                };

                Frame::data(frame)
            });
            Ok(Response::new(frame_stream.boxed()))
        }
        //Buffered response
        (&Method::POST, "/echo/reversed") => {
            let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
            if upper > 1024 * 64 {
                let mut resp = Response::new(full("Body too big"));
                *resp.status_mut() = hyper::StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(resp);
            }

            let whole_body = req.collect().await?.to_bytes();

            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();

            Ok(Response::new(full(reversed_body)))
        }

        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

// As defined by Hyper docs
//START
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>
        ::new()
        .map_err(|never| {
            match never {
            }
        })
        .boxed()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| {
            match never {
            }
        })
        .boxed()
}
//END

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
            eprintln!("Connection error: {}",e);
        }
    });

    let rows = big_client.query("SELECT * FROM tester", &[]).await?;

    let value: &str = rows[0].get(0);
    println!("{}",value);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new().serve_connection(io, service_fn(echo)).await {
                eprintln!("Error serving connection {:?}", err);
            }
        });
    }
}
