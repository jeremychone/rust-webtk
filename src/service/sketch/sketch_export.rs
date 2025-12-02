use crate::service::sketch::list_artboards;
use crate::support::files::{self, looks_like_file_path};
use crate::support::{strings, xmls};
use crate::{Error, Result};
use simple_fs::{SPath, ensure_dir, read_to_string};
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

	// Check if svg-symbols format is requested
	let has_svg_symbols = formats.contains(&"svg-symbols");

	// Separate svg-symbols from regular formats
	let regular_formats: Vec<&str> = formats.iter().filter(|f| **f != "svg-symbols").copied().collect();

	let mut exported_files = Vec::new();

	// Handle svg-symbols export
	if has_svg_symbols {
		let symbols_files = export_svg_symbols(sketch_file, &artboards, output_path)?;
		exported_files.extend(symbols_files);
	}

	// Handle regular formats
	if !regular_formats.is_empty() {
		let regular_files = export_regular_formats(sketch_file, &artboards, &regular_formats, output_path)?;
		exported_files.extend(regular_files);
	}

	Ok(exported_files)
}

/// Exports artboards as SVG symbols into a single SVG file.
fn export_svg_symbols(
	sketch_file: &SPath,
	artboards: &[crate::service::sketch::Artboard],
	output_path: &SPath,
) -> Result<Vec<String>> {
	// Determine the target file path
	let target_file = if looks_like_file_path(output_path) {
		output_path.clone()
	} else {
		// It's a directory, use symbols.svg as filename
		output_path.join("symbols.svg")
	};

	// Create a cache directory for temporary SVG exports
	let cache_dir = target_file.parent().unwrap_or_else(|| SPath::new(".")).join(".cache-symbols");

	ensure_dir(cache_dir.as_std_path())
		.map_err(|e| format!("Failed to create cache directory '{}': {e}", cache_dir))?;

	// Build the items argument (comma-separated UIDs)
	let item_ids: Vec<&str> = artboards.iter().map(|ab| ab.uid.as_str()).collect();
	let items_arg = item_ids.join(",");

	// Export SVGs to cache directory
	let output = Command::new(SKETCHTOOL_PATH)
		.arg("--format=svg")
		.arg("--include-symbols=YES")
		.arg(format!("--items={items_arg}"))
		.arg(format!("--output={}", cache_dir.as_str()))
		.arg("export")
		.arg("artboards")
		.arg(sketch_file.as_str())
		.output()
		.map_err(|e| format!("Failed to execute sketchtool: {e}"))?;

	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr);
		let _ = fs::remove_dir_all(cache_dir.as_std_path());
		return Err(format!("sketchtool export failed for svg-symbols: {stderr}").into());
	}

	// Collect all exported SVG files
	let svg_files =
		simple_fs::list_files(cache_dir.as_std_path(), Some(&["**/*.svg"]), None).map_err(Error::custom_from_err)?;

	// Build symbols from exported SVGs
	let mut symbols = Vec::new();
	for (idx, svg_file) in svg_files.iter().enumerate() {
		let artboard_name = &artboards[idx].name;
		let symbol_id = strings::canonicalize_name(artboard_name);

		let svg_content = read_to_string(svg_file.path()).map_err(Error::custom_from_err)?;

		if let Some(symbol) = convert_svg_to_symbol(&svg_content, &symbol_id) {
			symbols.push(symbol);
		}
	}

	// Build the combined SVG symbols file
	let symbols_content = build_svg_symbols_file(&symbols);

	// Ensure target parent directory exists
	if let Some(parent) = target_file.parent() {
		ensure_dir(parent.as_std_path()).map_err(|e| format!("Failed to create parent directory '{}': {e}", parent))?;
	}

	// Write the symbols file
	fs::write(target_file.as_std_path(), symbols_content)
		.map_err(|e| format!("Failed to write symbols file '{}': {e}", target_file))?;

	// Clean up cache directory
	let _ = fs::remove_dir_all(cache_dir.as_std_path());

	Ok(vec![target_file.to_string()])
}

/// Converts an SVG file content to a symbol element.
fn convert_svg_to_symbol(svg_content: &str, symbol_id: &str) -> Option<String> {
	// Extract viewBox from the SVG using quick-xml
	let viewbox = xmls::extract_root_attribute(svg_content, "viewBox")?;

	// Extract the inner content (everything between <svg ...> and </svg>)
	let inner_content = xmls::extract_root_inner_content(svg_content)?;

	// Canonicalize all id attributes within the inner content using quick-xml
	let inner_content = xmls::transform_id_attributes(&inner_content, strings::canonicalize_name);

	// Indent the inner content for proper formatting
	let indented_content = indent_content(&inner_content, 4);

	Some(format!(
		r#"  <symbol id="{symbol_id}" viewBox="{viewbox}">
{indented_content}
  </symbol>"#
	))
}

/// Indents each line of content by the specified number of spaces.
/// First removes common leading whitespace, then applies the new base indentation
/// while preserving relative indentation between lines.
fn indent_content(content: &str, base_spaces: usize) -> String {
	if content.is_empty() {
		return String::new();
	}

	// Find the minimum indentation among non-empty lines
	let min_indent = content
		.lines()
		.filter(|line| !line.trim().is_empty())
		.map(|line| line.len() - line.trim_start().len())
		.min()
		.unwrap_or(0);

	let base_indent = " ".repeat(base_spaces);
	content
		.lines()
		.map(|line| {
			if line.trim().is_empty() {
				String::new()
			} else {
				// Calculate this line's indentation relative to min_indent
				let line_indent = line.len() - line.trim_start().len();
				let relative_indent = line_indent.saturating_sub(min_indent);
				let extra_indent = " ".repeat(relative_indent);
				let trimmed = line.trim_start();
				format!("{base_indent}{extra_indent}{trimmed}")
			}
		})
		.collect::<Vec<_>>()
		.join("\n")
}

/// Builds the combined SVG symbols file.
fn build_svg_symbols_file(symbols: &[String]) -> String {
	let mut result = String::new();
	result.push_str(r#"<svg width="0" height="0" style="position:absolute">"#);
	result.push('\n');

	for (idx, symbol) in symbols.iter().enumerate() {
		// Add empty line before symbols, except for the first one
		if idx > 0 {
			result.push('\n');
		}
		result.push_str(symbol);
		result.push('\n');
	}

	result.push_str("</svg>\n");
	result
}

/// Exports artboards using regular sketchtool formats (svg, png, jpeg).
fn export_regular_formats(
	sketch_file: &SPath,
	artboards: &[crate::service::sketch::Artboard],
	formats: &[&str],
	output_path: &SPath,
) -> Result<Vec<String>> {
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
