mod cli;
mod environment;
mod handler;
mod process;
mod server;

use bytes::Bytes;
use std::sync::Arc;
use std::time::Instant;
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

#[derive(Serialize)]
struct Health {
    status: String,
    pid: u32,
}

#[derive(Serialize)]
struct Handlers {
    handlers: Vec<String>,
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

fn health() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("health"))
        .map(|| warp::reply::json(&Health {
            status: "Ok".to_string(),
            pid: std::process::id(),
        }))
}

fn handlers(environment: Arc<environment::Environment>)
            -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("handlers"))
        .map(move || match handler::configured_handlers(&environment) {
            Ok(keys) => warp::reply::with_status(
                warp::reply::json(&Handlers { handlers: keys }),
                StatusCode::OK),
            Err(e) => warp::reply::with_status(
                warp::reply::json(&Response {
                    status: "Err".to_string(),
                    code: -1,
                    stdout: "".to_string(),
                    stderr: e,
                }),
                StatusCode::INTERNAL_SERVER_ERROR),
        })
}

fn notify(environment: Arc<environment::Environment>)
          -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path!("notify" / String))
        .and(body_size_limit())
        .and(warp::body::bytes())
        .then(move |name: String, body: Bytes| {
            let environment = environment.clone();
            async move {
                if !server::is_valid_key(&name) {
                    eprintln!("[WARN] rejected key {:?}", name);
                    return warp::reply::with_status(
                        warp::reply::json(&Response {
                            status: "Err".to_string(),
                            code: -1,
                            stdout: "".to_string(),
                            stderr: "The Key should contain only alphanumeric characters".to_string(),
                        }),
                        StatusCode::BAD_REQUEST);
                }
                let started = Instant::now();
                let result = handler::execute(&environment, name.clone(), body).await;
                let elapsed = started.elapsed().as_millis();
                match result {
                    Ok(ok) => {
                        println!("{} code={} in {}ms", name, ok.code(), elapsed);
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
                        eprintln!("[ERROR] {} failed in {}ms: {}", name, elapsed, e);
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
            }
        })
}

fn routes(environment: Arc<environment::Environment>)
          -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    notify(environment.clone())
        .or(health())
        .or(handlers(environment))
        .recover(handle_rejection)
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut terminate = match signal(SignalKind::terminate()) {
            Ok(stream) => stream,
            Err(_) => return std::future::pending().await,
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

async fn entry_point() -> Result<(), String> {
    let arguments = cli::arguments();
    match arguments.command() {
        Command::Server(server) => {
            let port_number = server.port()?;
            let bind_address = server.bind()?;
            if !bind_address.is_loopback() {
                eprintln!("[WARN] {} is not loopback: ntfd has no authentication and will run handlers for anyone who can reach it",
                          bind_address);
            }
            // resolved once at startup so that a broken environment fails
            // loudly here instead of on every request
            let environment = Arc::new(environment::system_environment()?);

            let server = warp::serve(routes(environment))
                .bind((bind_address, port_number))
                .await
                .graceful(shutdown_signal());

            println!("Started {{pid={} address={} port={}}}",
                     std::process::id(), bind_address, port_number);
            server.run().await;
            println!("Stopped");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn env_for(dir: PathBuf) -> Arc<environment::Environment> {
        Arc::new(environment::Environment::for_dir(dir))
    }

    #[cfg(unix)]
    fn write_handler(dir: &std::path::Path, key: &str, script: &str) {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        let handler_dir = dir.join("conf").join(key);
        std::fs::create_dir_all(&handler_dir).unwrap();
        let run = handler_dir.join("run");
        let mut file = std::fs::File::create(&run).unwrap();
        file.write_all(script.as_bytes()).unwrap();
        drop(file);
        std::fs::set_permissions(&run, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[tokio::test]
    async fn invalid_key_is_answered_with_400() {
        let dir = tempfile::tempdir().unwrap();
        let response = warp::test::request()
            .method("POST")
            .path("/notify/bad-key")
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::BAD_REQUEST, response.status());
    }

    #[tokio::test]
    async fn missing_handler_is_answered_with_500() {
        let dir = tempfile::tempdir().unwrap();
        let response = warp::test::request()
            .method("POST")
            .path("/notify/nosuch")
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    }

    #[tokio::test]
    async fn oversized_body_is_answered_with_413() {
        let dir = tempfile::tempdir().unwrap();
        let body = vec![b'a'; (MAX_BODY_SIZE + 1) as usize];
        let response = warp::test::request()
            .method("POST")
            .path("/notify/ping")
            .body(body)
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::PAYLOAD_TOO_LARGE, response.status());
    }

    #[tokio::test]
    async fn get_is_not_allowed() {
        let dir = tempfile::tempdir().unwrap();
        let response = warp::test::request()
            .method("GET")
            .path("/notify/ping")
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::METHOD_NOT_ALLOWED, response.status());
    }

    #[tokio::test]
    async fn unknown_path_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let response = warp::test::request()
            .method("POST")
            .path("/nope")
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    async fn health_reports_ok() {
        let dir = tempfile::tempdir().unwrap();
        let response = warp::test::request()
            .method("GET")
            .path("/health")
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::OK, response.status());
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert_eq!("Ok", body["status"]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn handlers_lists_the_configured_keys() {
        let dir = tempfile::tempdir().unwrap();
        write_handler(dir.path(), "ping", "#!/bin/sh
echo pong
");
        write_handler(dir.path(), "default", "#!/bin/sh
echo fallback
");
        let response = warp::test::request()
            .method("GET")
            .path("/handlers")
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::OK, response.status());
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(serde_json::json!(["default", "ping"]), body["handlers"]);
    }

    #[tokio::test]
    async fn handlers_is_empty_without_a_conf_directory() {
        let dir = tempfile::tempdir().unwrap();
        let response = warp::test::request()
            .method("GET")
            .path("/handlers")
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::OK, response.status());
        let body: serde_json::Value = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(serde_json::json!([]), body["handlers"]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn successful_handler_is_answered_with_200_and_its_output() {
        let dir = tempfile::tempdir().unwrap();
        write_handler(dir.path(), "ping", "#!/bin/sh\necho pong\n");
        let response = warp::test::request()
            .method("POST")
            .path("/notify/ping")
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::OK, response.status());
        let body: serde_json::Value =
            serde_json::from_slice(response.body()).unwrap();
        assert_eq!("Ok", body["status"]);
        assert_eq!(0, body["code"]);
        assert_eq!("pong\n", body["stdout"]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn failing_handler_reports_its_exit_code() {
        let dir = tempfile::tempdir().unwrap();
        write_handler(dir.path(), "ping", "#!/bin/sh\necho oops >&2\nexit 3\n");
        let response = warp::test::request()
            .method("POST")
            .path("/notify/ping")
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::BAD_REQUEST, response.status());
        let body: serde_json::Value =
            serde_json::from_slice(response.body()).unwrap();
        assert_eq!(3, body["code"]);
        assert_eq!("oops\n", body["stderr"]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn json_fields_reach_the_handler_as_env_vars() {
        let dir = tempfile::tempdir().unwrap();
        write_handler(dir.path(), "ping",
                      "#!/bin/sh\necho \"$NTFD_JSON_FIELD_TITLE/$NTFD_JSON_FIELD_N\"\n");
        let response = warp::test::request()
            .method("POST")
            .path("/notify/ping")
            .body(r#"{"title":"hi","n":7}"#)
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::OK, response.status());
        let body: serde_json::Value =
            serde_json::from_slice(response.body()).unwrap();
        assert_eq!("hi/7\n", body["stdout"]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn default_handler_is_used_for_an_unknown_key() {
        let dir = tempfile::tempdir().unwrap();
        write_handler(dir.path(), "default", "#!/bin/sh\necho fallback\n");
        let response = warp::test::request()
            .method("POST")
            .path("/notify/whatever")
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::OK, response.status());
        let body: serde_json::Value =
            serde_json::from_slice(response.body()).unwrap();
        assert_eq!("fallback\n", body["stdout"]);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn malformed_json_is_answered_with_500() {
        let dir = tempfile::tempdir().unwrap();
        write_handler(dir.path(), "ping", "#!/bin/sh\necho pong\n");
        let response = warp::test::request()
            .method("POST")
            .path("/notify/ping")
            .body("not json")
            .reply(&routes(env_for(dir.path().to_path_buf())))
            .await;
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, response.status());
    }
}
