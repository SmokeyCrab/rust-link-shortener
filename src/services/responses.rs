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
	Ok(Response::new(hyper_template_funcs::full(
		Bytes::from(std::fs::read("assets/favicon.svg").expect("Favicon could not be opened"))
	)))
}