mod cli;
mod environment;
mod handler;
mod process;

use warp::Filter;
use cli::Command;
use serde::{Serialize};
use warp::http::StatusCode;

#[derive(Serialize)]
struct Response {
    status: String,
    code: i32,
    stdout: String,
    stderr: String,
}

async fn entry_point() -> Result<(), String> {
    let arguments = cli::arguments();
    match arguments.command() {
        Command::Server(server) => {
            let port_number = server.port()?;
            let hello = warp::post()
                .and(warp::path!("notify" / String))
                .and(warp::body::bytes())
                .map(|name, body| {
                    let environment = environment::system_environment();
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
            println!("Started {{pid={} port={}}}", std::process::id(), port_number);
            warp::serve(hello)
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