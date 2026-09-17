mod cli;

use clap::Parser;
use cli::args::{Args, Command};

fn main() {
    let result = match Args::parse().command {
        Command::Embed {
            image,
            payload,
            output,
        } => cli::commands::embed::run(&image, &payload, &output),
        Command::Extract { image, output } => cli::commands::extract::run(&image, &output),
        Command::Inspect { image } => cli::commands::inspect::run(&image),
        Command::Verify { image } => cli::commands::verify::run(&image),
    };
    if let Err(error) = result {
        eprintln!("vessel: {error}");
        std::process::exit(1);
    }
}
