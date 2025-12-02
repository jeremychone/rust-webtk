/// Canonicalizes a name by replacing non-alphanumeric characters with dashes
/// and collapsing consecutive dashes into a single dash.
pub fn canonicalize_name(name: &str) -> String {
	let mut result = String::with_capacity(name.len());
	let mut prev_was_dash = false;

	for ch in name.chars() {
		if ch.is_ascii_alphanumeric() {
			result.push(ch);
			prev_was_dash = false;
		} else if !prev_was_dash {
			result.push('-');
			prev_was_dash = true;
		}
		// else: consecutive non-alphanumeric, skip
	}

	// Trim leading and trailing dashes
	result.trim_matches('-').to_string()
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_support_strings_canonicalize_name_simple() -> Result<()> {
		// -- Setup & Fixtures
		let input = "ico/user/fill";

		// -- Exec
		let result = canonicalize_name(input);

		// -- Check
		assert_eq!(result, "ico-user-fill");

		Ok(())
	}

	#[test]
	fn test_support_strings_canonicalize_name_with_consecutive() -> Result<()> {
		// -- Setup & Fixtures
		let input = "ico//user--fill";

		// -- Exec
		let result = canonicalize_name(input);

		// -- Check
		assert_eq!(result, "ico-user-fill");

		Ok(())
	}

	#[test]
	fn test_support_strings_canonicalize_name_with_special_chars() -> Result<()> {
		// -- Setup & Fixtures
		let input = "my@icon#name!test";

		// -- Exec
		let result = canonicalize_name(input);

		// -- Check
		assert_eq!(result, "my-icon-name-test");

		Ok(())
	}

	#[test]
	fn test_support_strings_canonicalize_name_with_leading_trailing() -> Result<()> {
		// -- Setup & Fixtures
		let input = "/ico/user/";

		// -- Exec
		let result = canonicalize_name(input);

		// -- Check
		assert_eq!(result, "ico-user");

		Ok(())
	}
}

// endregion: --- Tests
