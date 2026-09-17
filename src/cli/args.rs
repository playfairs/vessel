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
    Create {
        image: PathBuf,
        payload: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
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
    Run {
        image: PathBuf,
        #[arg(trailing_var_arg = true)]
        arguments: Vec<String>,
    },
}
