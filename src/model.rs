use std::net::SocketAddr;

use clap::Parser;

#[derive(Debug, Parser)]
pub enum Cli {
    Server { port: u16 },
    Client { host: SocketAddr },
}
