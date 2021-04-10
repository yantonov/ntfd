mod cli;

use warp::Filter;
use cli::Command;

async fn entry_point() -> Result<(), String> {
    let arguments = cli::arguments();
    match arguments.command() {
        Command::Server(server) => {
            let port_number = server.port()?;
            // GET /hello/warp => 200 OK with body "Hello, warp!"
            let hello = warp::path!("hello" / String)
                .map(|name| format!("Hello, {}!", name));

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