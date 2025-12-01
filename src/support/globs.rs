use crate::Result;
use globset::GlobSet;
use simple_fs::get_glob_set;

/// Builds a GlobSet from an optional slice of pattern strings.
/// Returns None if no patterns are provided.
pub fn build_glob_set(patterns: Option<&[&str]>) -> Result<Option<GlobSet>> {
	match patterns {
		Some(globs) if !globs.is_empty() => {
			let set = get_glob_set(globs)
				.map_err(|e| format!("Invalid glob pattern(s): {globs:?}. Error: {e}"))
				.map_err(crate::Error::custom)?;
			Ok(Some(set))
		}
		_ => Ok(None),
	}
}

/// Checks if a value matches the glob set (if present).
/// Returns true if no glob set is provided.
pub fn matches_glob_set(glob_set: Option<&GlobSet>, value: &str) -> bool {
	glob_set.as_ref().is_none_or(|gs| gs.is_match(value))
}
