// region:    --- Modules

mod error;

pub use error::{Error, Result};

// endregion: --- Modules

fn main() -> Result<()> {
	println!("Hello webtk world!");

	Ok(())
}
