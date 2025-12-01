use clap::Parser;

// region:    --- Modules

mod cli;
mod error;

pub use error::{Error, Result};

// endregion: --- Modules

fn main() -> Result<()> {
	let _args = cli::args::CliArgs::parse();

	println!("Hello webtk world!");

	Ok(())
}
