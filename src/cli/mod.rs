use clap::Parser;

#[derive(Parser)]
#[clap(version)]
struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
pub enum Command {
    #[clap(about = "run notification server", display_order = 0)]
    Server(Server)
}

#[derive(Parser)]
pub struct Server {
    #[clap(help = "port number (default port = 4242)", short, long)]
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
    Arguments { args: Opts::parse() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_port_is_4242() {
        let server = Server { port: None };
        assert_eq!(Ok(4242), server.port());
    }

    #[test]
    fn explicit_port_is_returned() {
        let server = Server { port: Some(8080) };
        assert_eq!(Ok(8080), server.port());
    }

    #[test]
    fn boundary_min_port_1024_is_valid() {
        let server = Server { port: Some(1024) };
        assert_eq!(Ok(1024), server.port());
    }

    #[test]
    fn boundary_max_port_65535_is_valid() {
        let server = Server { port: Some(65535) };
        assert_eq!(Ok(65535), server.port());
    }

    #[test]
    fn port_1023_is_rejected() {
        let server = Server { port: Some(1023) };
        assert!(server.port().is_err());
    }

    #[test]
    fn port_zero_is_rejected() {
        let server = Server { port: Some(0) };
        assert!(server.port().is_err());
    }
}