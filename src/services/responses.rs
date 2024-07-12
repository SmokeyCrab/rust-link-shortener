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

use hex;

use hyper::header::{ LOCATION };

use random_string::generate;

pub mod hyper_template_funcs;

use crate::db;
use crate::config;
use crate::connection;

pub fn fail() -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let mut not_found = Response::new(hyper_template_funcs::empty());
    *not_found.status_mut() = StatusCode::NOT_FOUND;
    Ok(not_found)
}

pub fn home(
    host_config: Arc<config::HostConfig>
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    Ok(
        Response::new(
            hyper_template_funcs::full(
                sprintf!(
                    "Welcome to the URL Shortener, POST a URL to \"%s/\"",
                    host_config.host_url
                ).unwrap()
            )
        )
    )
}

pub fn favicon() -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    Ok(
        Response::new(
            hyper_template_funcs::full(
                Bytes::from(
                    std::fs::read("assets/favicon.svg").expect("Favicon could not be opened")
                )
            )
        )
    )
}

pub async fn retrieve(
    pg_client: Arc<tokio_postgres::Client>,
    pg_config: Arc<config::PostgresConfig>,
    url: &str
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let upper_url = url.to_uppercase();

    let mut chars = upper_url.as_str().chars();
    chars.next();

    let rows = pg_client
        .query(
            &std::fs
                ::read_to_string("sql/retrieve.sql")
                .expect("Couldn't open request")
                .replace("{1}", chars.as_str())[..],
            &[]
        ).await
        .expect("SQL query failed");
    if rows.len() == 0 {
        return Ok(fail()?);
    }
    let value: &str = rows[0].get(0);
    println!("{}", value);
    let mut redirect = Response::new(hyper_template_funcs::empty());
    *redirect.status_mut() = StatusCode::FOUND; // 302
    redirect
        .headers_mut()
        .insert(
            LOCATION,
            String::from_utf8(hex::decode(value).expect("HEX decoder failed"))
                .expect("Invalid bytes provided by hex decoder")
                .parse()
                .unwrap()
        );

    Ok(redirect)
}

pub async fn write(
    pg_client: Arc<tokio_postgres::Client>,
    host_config: Arc<config::HostConfig>,
    pg_config: Arc<config::PostgresConfig>,
    req: Request<hyper::body::Incoming>
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // Buffer full request (Based on Hyper Example docs /echo/reversed)

    let upper = req.body().size_hint().upper().unwrap_or(u64::MAX);
    if upper > 256 {
        // 256 Characters max
        let mut resp = Response::new(hyper_template_funcs::full("Payload too large"));
        *resp.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
        return Ok(resp);
    }

    let url = String::from_utf8(
        req.collect().await?.to_bytes().iter().cloned().collect::<Vec<u8>>()
    ).expect("Something invalid sent");

    let charset = "QWERTYUIOPASDFGHJKLZXCVBNM1234567890";

    let mut new_id: String;

    while (
        {
            new_id = generate(5, charset);
            let rows1 = pg_client
                .query(
                    &std::fs
                        ::read_to_string("sql/retrieve.sql")
                        .expect("Couldn't open request")
                        .replace("{1}", &new_id[..])[..],
                    &[]
                ).await
                .expect("SQL query failed");
            rows1.len() != 0
        }
    ) {}
    println!("{} {}", new_id, url);

    let _rows2 = pg_client
        .query(
            &std::fs
                ::read_to_string("sql/write.sql")
                .expect("Couldn't open request")
                .replace("{1}", &new_id[..])
                .replace("{2}", &hex::encode(&url[..])[..])[..],
            &[]
        ).await
        .expect("SQL query failed");

    Ok(
        Response::new(
            hyper_template_funcs::full(sprintf!("%s/%s", host_config.host_url, new_id).unwrap())
        )
    )
}
