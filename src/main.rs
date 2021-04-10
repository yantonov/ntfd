mod cli;
mod environment;
mod handler;
mod process;

use warp::Filter;
use cli::Command;
use serde::{Serialize};

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
            let hello = warp::path!("notify" / String)
                .map(|name| {
                    let environment = environment::system_environment();
                    let result = handler::execute(&environment, name);
                    match result {
                        Ok(ok) => {
                            warp::reply::json(&Response {
                                status: "Ok".to_string(),
                                code: ok.code(),
                                stdout: ok.stdout().to_string(),
                                stderr: ok.stderr().to_string(),
                            })
                        }
                        Err(e) => {
                            warp::reply::json(&Response {
                                status: "Err".to_string(),
                                code: -1,
                                stdout: "".to_string(),
                                stderr: e.to_string(),
                            })
                        }
                    }
                });

            Ok(warp::serve(hello)
                .run(([127, 0, 0, 1], port_number))
                .await)
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