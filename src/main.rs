// region:    --- Modules

mod cli;
mod error;
mod handlers;
mod support;

pub use error::{Error, Result};

// endregion: --- Modules

fn main() {
	let res = cli::execute();

	if let Err(err) = res {
		eprintln!("Error: {err}");
		std::process::exit(1);
	}
}
