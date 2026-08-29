mod cli;
mod environment;
mod handler;
mod process;
mod server;

use bytes::Bytes;
use std::sync::Arc;
use warp::Filter;
use cli::Command;
use serde::{Serialize};
use warp::http::StatusCode;

/// The body only carries notification metadata, there is no reason
/// to buffer more than this in memory.
const MAX_BODY_SIZE: u64 = 64 * 1024;

#[derive(Serialize)]
struct Response {
    status: String,
    code: i32,
    stdout: String,
    stderr: String,
}

#[derive(Debug)]
struct BodyTooLarge;

impl warp::reject::Reject for BodyTooLarge {}

/// Rejects oversized bodies before they are buffered into memory.
/// A request without a content-length header is let through: curl sends
/// none for a bodyless POST, and warp's own content_length_limit answers
/// those with 411 instead.
fn body_size_limit() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    warp::header::optional::<u64>("content-length")
        .and_then(|length: Option<u64>| async move {
            match length {
                Some(length) if length > MAX_BODY_SIZE => {
                    Err(warp::reject::custom(BodyTooLarge))
                }
                _ => Ok(()),
            }
        })
        .untuple_one()
}

async fn handle_rejection(rejection: warp::Rejection)
                          -> Result<impl warp::Reply, warp::Rejection> {
    if rejection.find::<BodyTooLarge>().is_some() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&Response {
                status: "Err".to_string(),
                code: -1,
                stdout: "".to_string(),
                stderr: format!("body is larger than {} bytes", MAX_BODY_SIZE),
            }),
            StatusCode::PAYLOAD_TOO_LARGE));
    }
    Err(rejection)
}

async fn entry_point() -> Result<(), String> {
    let arguments = cli::arguments();
    match arguments.command() {
        Command::Server(server) => {
            let port_number = server.port()?;
            // resolved once at startup so that a broken environment fails
            // loudly here instead of on every request
            let environment = Arc::new(environment::system_environment()?);

            let hello = warp::post()
                .and(warp::path!("notify" / String))
                .and(body_size_limit())
                .and(warp::body::bytes())
                .map(move |name: String, body: Bytes| {
                    let check_key = server::is_valid_key();
                    if !check_key(&name) {
                        return warp::reply::with_status(
                            warp::reply::json(&Response {
                                status: "Err".to_string(),
                                code: -1,
                                stdout: "".to_string(),
                                stderr: "The Key should contain only alphanumeric characters".to_string(),
                            }),
                            StatusCode::INTERNAL_SERVER_ERROR);
                    }
                    let result = handler::execute(&environment, name, body);
                    match result {
                        Ok(ok) => {
                            let status_text = if ok.code() == 0 {
                                "Ok"
                            } else {
                                "Err"
                            };
                            let http_status = if ok.code() == 0 {
                                StatusCode::OK
                            } else {
                                StatusCode::BAD_REQUEST
                            };
                            warp::reply::with_status(
                                warp::reply::json(&Response {
                                    status: status_text.to_string(),
                                    code: ok.code(),
                                    stdout: ok.stdout().to_string(),
                                    stderr: ok.stderr().to_string(),
                                }),
                                http_status)
                        }
                        Err(e) => {
                            warp::reply::with_status(
                                warp::reply::json(&Response {
                                    status: "Err".to_string(),
                                    code: -1,
                                    stdout: "".to_string(),
                                    stderr: e,
                                }),
                                StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }
                });
            let routes = hello.recover(handle_rejection);

            println!("Started {{pid={} port={}}}", std::process::id(), port_number);
            warp::serve(routes)
                .run(([127, 0, 0, 1], port_number))
                .await;
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() {
    match entry_point().await {
        Ok(_) => std::process::exit(0),
        Err(message) => {
            eprintln!("[ERROR] {}", message);
            std::process::exit(1);
        }
    }
}