use clap::{Clap, crate_version};

#[derive(Clap)]
#[clap(version = crate_version ! ())]
struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clap)]
pub enum Command {
    #[clap(about = "run notification server", display_order = 0)]
    Server(Server)
}

#[derive(Clap)]
pub struct Server {
    #[clap(about = "port number (default port = 4242)", short, long, )]
    port: Option<u16>,
}

impl Server {
    pub fn port(&self) -> Result<u16, String> {
        let default_port: u16 = 4242;
        let port: u16 = self.port
            .unwrap_or(default_port);
        let min_port = 1024;
        let max_port = 65535;
        if port < min_port || port > max_port {
            return Err(format!("Port number should be between {} and {}", min_port, max_port));
        }
        Ok(port)
    }
}

pub struct Arguments {
    args: Opts
}

impl Arguments {
    pub fn command(&self) -> &Command {
        &self.args.command
    }
}

pub fn arguments() -> Arguments {
    return Arguments { args: Opts::parse() };
}