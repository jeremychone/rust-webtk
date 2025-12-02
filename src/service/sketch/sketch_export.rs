use crate::service::sketch::list_artboards;
use crate::support::files::{self, looks_like_file_path};
use crate::support::strings;
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
	// Extract viewBox from the SVG
	let viewbox = extract_svg_attribute(svg_content, "viewBox")?;

	// Extract the inner content (everything between <svg ...> and </svg>)
	let inner_content = extract_svg_inner_content(svg_content)?;

	// Canonicalize all id attributes within the inner content
	let inner_content = canonicalize_svg_ids(&inner_content);

	Some(format!(
		r#"  <symbol id="{symbol_id}" viewBox="{viewbox}">
{inner_content}
  </symbol>"#
	))
}

/// Canonicalizes all id attribute values in SVG content.
/// Replaces special characters with dashes using the same canonicalization as symbol IDs.
fn canonicalize_svg_ids(content: &str) -> String {
	let mut result = String::with_capacity(content.len());
	let mut remaining = content;

	while !remaining.is_empty() {
		// Look for id=" or id='
		if let Some(idx) = remaining.find("id=\"") {
			// Copy everything before the id attribute
			result.push_str(&remaining[..idx]);
			result.push_str("id=\"");
			remaining = &remaining[idx + 4..];

			// Find the closing quote
			if let Some(end_idx) = remaining.find('"') {
				let id_value = &remaining[..end_idx];
				let canonicalized = strings::canonicalize_name(id_value);
				result.push_str(&canonicalized);
				result.push('"');
				remaining = &remaining[end_idx + 1..];
			} else {
				// Malformed, just copy the rest
				result.push_str(remaining);
				break;
			}
		} else if let Some(idx) = remaining.find("id='") {
			// Copy everything before the id attribute
			result.push_str(&remaining[..idx]);
			result.push_str("id='");
			remaining = &remaining[idx + 4..];

			// Find the closing quote
			if let Some(end_idx) = remaining.find('\'') {
				let id_value = &remaining[..end_idx];
				let canonicalized = strings::canonicalize_name(id_value);
				result.push_str(&canonicalized);
				result.push('\'');
				remaining = &remaining[end_idx + 1..];
			} else {
				// Malformed, just copy the rest
				result.push_str(remaining);
				break;
			}
		} else {
			// No more id attributes, copy the rest
			result.push_str(remaining);
			break;
		}
	}

	result
}

/// Extracts an attribute value from an SVG tag.
fn extract_svg_attribute(svg_content: &str, attr_name: &str) -> Option<String> {
	let svg_start = svg_content.find("<svg")?;
	let svg_tag_end = svg_content[svg_start..].find('>')?;
	let svg_tag = &svg_content[svg_start..svg_start + svg_tag_end];

	// Look for attribute="value" or attribute='value'
	let attr_pattern = format!("{attr_name}=\"");
	if let Some(attr_start) = svg_tag.find(&attr_pattern) {
		let value_start = attr_start + attr_pattern.len();
		let value_end = svg_tag[value_start..].find('"')?;
		return Some(svg_tag[value_start..value_start + value_end].to_string());
	}

	let attr_pattern = format!("{attr_name}='");
	if let Some(attr_start) = svg_tag.find(&attr_pattern) {
		let value_start = attr_start + attr_pattern.len();
		let value_end = svg_tag[value_start..].find('\'')?;
		return Some(svg_tag[value_start..value_start + value_end].to_string());
	}

	None
}

/// Extracts the inner content of an SVG element.
fn extract_svg_inner_content(svg_content: &str) -> Option<String> {
	let svg_start = svg_content.find("<svg")?;
	let opening_tag_end = svg_content[svg_start..].find('>')? + svg_start + 1;
	let closing_tag_start = svg_content.rfind("</svg>")?;

	if opening_tag_end >= closing_tag_start {
		return Some(String::new());
	}

	let inner = &svg_content[opening_tag_end..closing_tag_start];

	// Trim leading/trailing whitespace to avoid empty lines at start/end
	let inner = inner.trim();

	if inner.is_empty() {
		return Some(String::new());
	}

	// Indent the inner content
	let indented: Vec<String> = inner.lines().map(|line| format!("    {line}")).collect();

	Some(indented.join("\n"))
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
