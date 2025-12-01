use crate::{Error, Result};
use simple_fs::SPath;

/// Checks if a file exists, returning Error::FileNotFound otherwise.
pub fn check_file_exists(path: &SPath) -> Result<()> {
	if !path.exists() {
		return Err(Error::FileNotFound(path.clone()));
	}
	Ok(())
}
