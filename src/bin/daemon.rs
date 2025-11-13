use netvisor::daemon::{
    shared::{
        storage::{ CliArgs },
    }
};
use uuid::Uuid;
use clap::Parser;

#[derive(Parser)]
#[command(name = "netvisor-daemon")]
#[command(about = "NetVisor network discovery and test execution daemon")]
struct Cli {
    /// Server target (IP or hostname)
    #[arg(long)]
    server_target: Option<String>,

    /// Server port
    #[arg(long)]
    server_port: Option<u16>,

    /// Network ID to join
    #[arg(long)]
    network_id: Option<String>,

    /// Daemon listen port
    #[arg(short, long)]
    daemon_port: Option<u16>,

    /// Daemon listen host
    #[arg(long)]
    host: Option<String>,

    /// Daemon name
    #[arg(long)]
    name: Option<String>,

    /// Log level
    #[arg(long)]
    log_level: Option<String>,

    /// Heartbeat interval in seconds
    #[arg(long)]
    heartbeat_interval: Option<u64>,

    /// Daemon bind address
    #[arg(long)]
    bind_address: Option<String>,

    /// Concurrent scans for discovery
    #[arg(long)]
    concurrent_scans: Option<usize>,

    /// API key
    #[arg(long)]
    daemon_api_key: Option<String>,

    /// Docker socket proxy
    #[arg(long)]
    docker_proxy: Option<String>
}

impl From<Cli> for CliArgs {
    fn from(cli: Cli) -> Self {
        Self {
            server_target: cli.server_target,
            server_port: cli.server_port,
            daemon_port: cli.daemon_port,
            name: cli.name,
            bind_address: cli.bind_address,
            network_id: cli.network_id.and_then(|s| Uuid::parse_str(&s).ok()),
            log_level: cli.log_level,
            heartbeat_interval: cli.heartbeat_interval,
            concurrent_scans: cli.concurrent_scans,
            daemon_api_key: cli.daemon_api_key,
            docker_proxy: cli.docker_proxy
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    tokio::signal::ctrl_c().await?;
    Ok(())
}