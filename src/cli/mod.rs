use clap::Parser;
use std::net::{IpAddr, Ipv4Addr};

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

    #[clap(help = "address to listen on (default = 127.0.0.1)", short, long)]
    bind: Option<String>,
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

    pub fn bind(&self) -> Result<IpAddr, String> {
        match &self.bind {
            None => Ok(IpAddr::V4(Ipv4Addr::LOCALHOST)),
            Some(address) => address
                .parse::<IpAddr>()
                .map_err(|_| format!("{} is not a valid ip address", address)),
        }
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

    fn server(port: Option<u16>, bind: Option<&str>) -> Server {
        Server { port, bind: bind.map(|v| v.to_string()) }
    }

    #[test]
    fn default_port_is_4242() {
        assert_eq!(Ok(4242), server(None, None).port());
    }

    #[test]
    fn default_bind_is_loopback() {
        assert_eq!(Ok(IpAddr::V4(Ipv4Addr::LOCALHOST)), server(None, None).bind());
    }

    #[test]
    fn explicit_bind_is_parsed() {
        assert_eq!(Ok(IpAddr::V4(Ipv4Addr::UNSPECIFIED)), server(None, Some("0.0.0.0")).bind());
    }

    #[test]
    fn ipv6_bind_is_parsed() {
        assert!(server(None, Some("::1")).bind().is_ok());
    }

    #[test]
    fn garbage_bind_is_rejected() {
        assert!(server(None, Some("not-an-address")).bind().is_err());
    }

    #[test]
    fn hostname_bind_is_rejected() {
        assert!(server(None, Some("localhost")).bind().is_err());
    }

    #[test]
    fn explicit_port_is_returned() {
        assert_eq!(Ok(8080), server(Some(8080), None).port());
    }

    #[test]
    fn boundary_min_port_1024_is_valid() {
        assert_eq!(Ok(1024), server(Some(1024), None).port());
    }

    #[test]
    fn boundary_max_port_65535_is_valid() {
        assert_eq!(Ok(65535), server(Some(65535), None).port());
    }

    #[test]
    fn port_1023_is_rejected() {
        assert!(server(Some(1023), None).port().is_err());
    }

    #[test]
    fn port_zero_is_rejected() {
        assert!(server(Some(0), None).port().is_err());
    }
}