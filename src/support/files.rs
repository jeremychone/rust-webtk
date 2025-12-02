use crate::{Error, Result};
use simple_fs::{SPath, SaferRemoveOptions};

/// Allowed substrings for directory deletion (safety check).
const DIR_DELETE_ALLOW_CONTAINS: &[&str] = &[".cache-symbols", ".cache"];

/// Allowed extensions for file deletion (safety check).
const FILE_DELETE_ALLOW_CONTAINS: &[&str] = &[".svg", ".png", ".jpeg"];

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

/// Safely deletes a directory if it passes safety checks.
/// Safety checks:
/// - The directory path must be below the current directory
/// - The directory path must contain one of the allowed substrings
///
/// Returns Ok(true) if the directory was deleted, Ok(false) if it didn't exist.
/// Returns an error if safety checks fail or deletion fails.
pub fn safer_delete_dir(dir_path: &SPath) -> Result<bool> {
	let options = SaferRemoveOptions::default().with_must_contain_any(DIR_DELETE_ALLOW_CONTAINS);

	simple_fs::safer_remove_dir(dir_path, options).map_err(Error::custom_from_err)
}

/// Safely deletes a file if it passes safety checks.
/// Safety checks:
/// - The file path must be below the current directory
/// - The file path must contain one of the allowed substrings
///
/// Returns Ok(true) if the file was deleted, Ok(false) if it didn't exist.
/// Returns an error if safety checks fail or deletion fails.
#[allow(unused)]
pub fn safer_delete_file(file_path: &SPath) -> Result<bool> {
	let options = SaferRemoveOptions::default().with_must_contain_any(FILE_DELETE_ALLOW_CONTAINS);

	simple_fs::safer_remove_file(file_path, options).map_err(Error::custom_from_err)
}
