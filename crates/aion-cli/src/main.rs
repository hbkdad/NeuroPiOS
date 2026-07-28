//! The `aion` CLI. Real functionality so far: a local multi-process
//! devnet (`aion devnet up|status|down`), per the original
//! `crates/aion-cli` README stub's target UX and docs/ROADMAP.md's Phase
//! 3+ scope. Node/job/model management subcommands remain not started --
//! they need a real job-execution/settlement pipeline (Phase 4/10) this
//! dev container doesn't have.

mod devnet;
mod worker;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

const DEFAULT_DEVNET_DIR: &str = ".aion-devnet";

#[derive(Parser)]
#[command(name = "aion", about = "AION node/devnet CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage a local multi-process development network.
    Devnet {
        #[command(subcommand)]
        action: DevnetAction,
    },
    /// Internal: runs one devnet node. Spawned by `devnet up` -- not
    /// meant to be invoked directly.
    #[command(hide = true)]
    Worker {
        #[arg(long)]
        index: usize,
        #[arg(long)]
        dir: PathBuf,
    },
}

#[derive(Subcommand)]
enum DevnetAction {
    /// Start a local devnet of the given number of nodes.
    Up {
        #[arg(long, default_value_t = 3)]
        nodes: usize,
        #[arg(long, default_value = DEFAULT_DEVNET_DIR)]
        dir: PathBuf,
    },
    /// Report each devnet node's liveness and known P2P connections.
    Status {
        #[arg(long, default_value = DEFAULT_DEVNET_DIR)]
        dir: PathBuf,
    },
    /// Stop every live devnet node and clear its records.
    Down {
        #[arg(long, default_value = DEFAULT_DEVNET_DIR)]
        dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Devnet { action } => match action {
            DevnetAction::Up { nodes, dir } => match devnet::up(nodes, &dir) {
                Ok(()) => {
                    println!("started {nodes} devnet node(s) in {dir:?}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            },
            DevnetAction::Status { dir } => match devnet::status(&dir) {
                Ok(rows) => {
                    if rows.is_empty() {
                        println!("no devnet nodes recorded in {dir:?}");
                    }
                    for (record, alive) in rows {
                        println!(
                            "node-{}: pid={} alive={} peer_id={} listen_addr={} connected_peers=[{}]",
                            record.index,
                            record.pid,
                            alive,
                            record.peer_id,
                            record.listen_addr,
                            record.connected_peers.join(", "),
                        );
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            },
            DevnetAction::Down { dir } => match devnet::down(&dir) {
                Ok(killed) => {
                    println!("stopped {killed} devnet node(s) in {dir:?}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    ExitCode::FAILURE
                }
            },
        },
        Command::Worker { index, dir } => match worker::run(index, dir).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("worker {index} failed: {e}");
                ExitCode::FAILURE
            }
        },
    }
}
