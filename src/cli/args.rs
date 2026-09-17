use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "vessel", version, about = "A vessel for executable payloads.")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Embed {
        image: PathBuf,
        payload: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    Extract {
        image: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    Inspect {
        image: PathBuf,
    },
    Verify {
        image: PathBuf,
    },
}
