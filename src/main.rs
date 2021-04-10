mod cli;
mod environment;
mod handler;
mod process;

use warp::Filter;
use cli::Command;

async fn entry_point() -> Result<(), String> {
    let arguments = cli::arguments();
    match arguments.command() {
        Command::Server(server) => {
            let port_number = server.port()?;
            // GET /hello/warp => 200 OK with body "Hello, warp!"
            let hello = warp::path!("hello" / String)
                .map(|name| {
                    let environment = environment::system_environment();
                    let result = handler::execute(&environment, name);
                    match result {
                        Ok(_) => {
                            format!("Ok!")
                        }
                        Err(e) => {
                            format!("[ERROR] {}", e)
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