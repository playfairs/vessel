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
        Command::Create {
            image,
            payload,
            output,
        } => cli::commands::create::run(&image, &payload, &output),
        Command::Extract { image, output } => cli::commands::extract::run(&image, &output),
        Command::Inspect { image } => cli::commands::inspect::run(&image),
        Command::Verify { image } => cli::commands::verify::run(&image),
        Command::Run { image, arguments } => match cli::commands::run::run(&image, &arguments) {
            Ok(code) => {
                std::process::exit(code);
            }
            Err(error) => Err(error),
        },
    };
    if let Err(error) = result {
        eprintln!("vessel: {error}");
        std::process::exit(1);
    }
}
