use crate::Result;
use crate::service::sketch::list_artboards;
use crate::support::files;
use simple_fs::{SPath, ensure_dir};
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
	let output_dir = output_dir.as_ref();

	files::check_file_exists(sketch_file)?;

	// Get artboards matching the glob patterns
	let artboards = list_artboards(sketch_file, glob_patterns)?;

	if artboards.is_empty() {
		return Ok(vec![]);
	}

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
				exported_files.push(line.to_string());
			}
		}
	}

	Ok(exported_files)
}
