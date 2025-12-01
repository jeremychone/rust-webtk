use crate::service::sketch::list_artboards;
use crate::support::files;
use crate::{Error, Result};
use simple_fs::{SPath, ensure_dir};
use std::fs;
use std::process::Command;

const SKETCHTOOL_PATH: &str = "/Applications/Sketch.app/Contents/Resources/sketchtool/bin/sketchtool";

/// Exports artboards from a Sketch file to the specified formats.
/// Returns a list of exported file paths.
pub fn export_artboards(
	sketch_file: impl AsRef<SPath>,
	glob_patterns: Option<&[&str]>,
	formats: &[&str],
	output_dir: impl AsRef<SPath>,
) -> Result<Vec<String>> {
	let sketch_file = sketch_file.as_ref();
	let output_path = output_dir.as_ref();

	files::check_file_exists(sketch_file)?;

	// Get artboards matching the glob patterns
	let artboards = list_artboards(sketch_file, glob_patterns)?;

	if artboards.is_empty() {
		return Ok(vec![]);
	}

	// Determine if output is a single file target
	let single_file_output = is_single_file_output(output_path, formats);

	// Validate single file output constraints
	if single_file_output {
		if artboards.len() > 1 {
			return Err(Error::custom(format!(
				"Output path '{}' is a file, but {} artboards matched. Use a directory for multiple exports.",
				output_path,
				artboards.len()
			)));
		}
		if formats.len() > 1 {
			return Err(Error::custom(format!(
				"Output path '{}' is a file, but {} formats specified. Use a directory for multiple formats.",
				output_path,
				formats.len()
			)));
		}
	}

	// Determine actual output directory (where sketchtool will write files)
	// For single file output, use a .cache subdirectory to capture sketchtool's output
	let (output_dir, cache_dir) = if single_file_output {
		let parent = output_path.parent().unwrap_or_else(|| SPath::new("."));
		let cache = parent.join(".cache");
		(cache.clone(), Some(cache))
	} else {
		(output_path.clone(), None)
	};

	// Ensure output directory exists
	ensure_dir(output_dir.as_std_path())
		.map_err(|e| format!("Failed to create output directory '{}': {e}", output_dir))?;

	// Build the items argument (comma-separated UIDs)
	let item_ids: Vec<&str> = artboards.iter().map(|ab| ab.uid.as_str()).collect();
	let items_arg = item_ids.join(",");

	let mut exported_files = Vec::new();

	// Export for each format
	for format in formats {
		let output = Command::new(SKETCHTOOL_PATH)
			.arg(format!("--format={format}"))
			.arg("--include-symbols=YES")
			.arg(format!("--items={items_arg}"))
			.arg(format!("--output={}", output_dir.as_str()))
			.arg("export")
			.arg("artboards")
			.arg(sketch_file.as_str())
			.output()
			.map_err(|e| format!("Failed to execute sketchtool: {e}"))?;

		if !output.status.success() {
			let stderr = String::from_utf8_lossy(&output.stderr);
			return Err(format!("sketchtool export failed for format '{format}': {stderr}").into());
		}

		// Parse stdout for exported file names
		let stdout = String::from_utf8_lossy(&output.stdout);
		for line in stdout.lines() {
			let line = line.trim();
			if !line.is_empty() {
				// If single file output, move the exported file from cache to the target path
				if let Some(ref cache) = cache_dir {
					// sketchtool outputs "Exported filename.ext" - we need to find the actual file in the cache
					// The file is placed in a subdirectory structure matching the artboard name path
					let exported_path = find_exported_file_in_cache(cache, format).ok_or("Cannot find exported")?;
					let target_path = output_path;

					// Ensure target parent directory exists
					if let Some(parent) = target_path.parent() {
						ensure_dir(parent.as_std_path())
							.map_err(|e| format!("Failed to create parent directory '{}': {e}", parent))?;
					}

					// Copy the file first (more reliable across filesystems), then remove source
					fs::copy(exported_path.as_std_path(), target_path.as_std_path())
						.map_err(|e| format!("Failed to copy exported file to '{}': {e}", target_path))?;

					// Clean up the cache directory (includes the source file)
					let _ = fs::remove_dir_all(cache.as_std_path());

					exported_files.push(target_path.to_string());
				} else {
					exported_files.push(line.to_string());
				}
			}
		}
	}

	Ok(exported_files)
}

/// Finds the first file with the given extension in the cache directory (recursively).
fn find_exported_file_in_cache(cache_dir: &SPath, format: &str) -> Option<SPath> {
	let pattern = format!("**/*.{format}");
	let files = simple_fs::list_files(cache_dir.as_std_path(), Some(&[pattern.as_str()]), None).ok()?;
	files.into_iter().next().map(|f| f.path().clone())
}

/// Checks if the output path appears to be a single file target.
/// Returns true if the path ends with an extension matching one of the export formats.
fn is_single_file_output(output_path: &SPath, formats: &[&str]) -> bool {
	let ext = output_path.ext();
	if ext.is_empty() {
		return false;
	}

	// Check if the extension matches any of the export formats
	let ext_lower = ext.to_lowercase();
	formats.iter().any(|f| f.to_lowercase() == ext_lower)
}
