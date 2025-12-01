use clap::Parser;
use cli::args::{CliArgs, Command, SketchCommand};

// region:    --- Modules

mod cli;
mod error;
mod service;
mod support;

pub use error::{Error, Result};

// endregion: --- Modules

fn main() {
	let args = CliArgs::parse();

	let Some(cmd) = args.command else {
		println!("Hello webtk world! Use --help for available commands.");
		return;
	};

	let res: Result<()> = match cmd {
		Command::Sketch { command } => match command {
			SketchCommand::ListArtboards { sketch_file } => cli::service_sketch::exec_list_artboards(&sketch_file),
		},
	};

	if let Err(err) = res {
		eprintln!("Error: {err}");
		std::process::exit(1);
	}
}
