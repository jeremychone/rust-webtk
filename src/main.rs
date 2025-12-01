use clap::Parser;
use cli::args::{CliArgs, Command, SketchCommand};

// region:    --- Modules

mod cli;
mod error;
mod service;

pub use error::{Error, Result};

// endregion: --- Modules

fn main() -> Result<()> {
	let args = CliArgs::parse();

	match args.command {
		Some(Command::Sketch { command }) => match command {
			SketchCommand::ListArtboards { sketch_file } => {
				cli::service_sketch::exec_list_artboards(&sketch_file)?;
			}
		},

		None => {
			println!("Hello webtk world! Use --help for available commands.");
		}
	}

	Ok(())
}
