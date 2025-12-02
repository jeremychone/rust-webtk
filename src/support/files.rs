use crate::{Error, Result};
use simple_fs::SPath;

/// Checks if a file exists, returning Error::FileNotFound otherwise.
pub fn check_file_exists(path: &SPath) -> Result<()> {
	if !path.exists() {
		return Err(Error::FileNotFound(path.clone()));
	}
	Ok(())
}

/// Returns true if the path looks like a file path (has an extension).
/// Returns false if it looks like a directory path (no extension).
pub fn looks_like_file_path(path: &SPath) -> bool {
	!path.ext().is_empty()
}
