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

	// Build symbols from exported SVGs, matching by artboard name
	let mut symbols = Vec::new();
	for artboard in artboards {
		let symbol_id = strings::canonicalize_name(&artboard.name);

		// Find the corresponding SVG file by matching the artboard name pattern
		// sketchtool exports files with names like "artboard-name.svg" where slashes become "/"
		let svg_file = find_svg_file_for_artboard(&cache_dir, &artboard.name)?;

		let svg_content = read_to_string(svg_file.path()).map_err(Error::custom_from_err)?;

		// Validate that the SVG content is not empty
		if svg_content.trim().is_empty() {
			let _ = fs::remove_dir_all(cache_dir.as_std_path());
			return Err(Error::custom(format!(
				"SVG file for artboard '{}' is empty: '{}'",
				artboard.name,
				svg_file.path()
			)));
		}

		let symbol = convert_svg_to_symbol(&svg_content, &symbol_id).ok_or_else(|| {
			// Clean up before returning error
			let _ = fs::remove_dir_all(cache_dir.as_std_path());
			Error::custom(format!(
				"Failed to convert SVG to symbol for artboard '{}': invalid SVG content. File: '{}', Content length: {} bytes",
				artboard.name,
				svg_file.path(),
				svg_content.len()
			))
		})?;

		// Validate that the symbol actually has content beyond just the opening/closing tags
		if !symbol.contains('<') || symbol.matches('<').count() <= 2 {
			let _ = fs::remove_dir_all(cache_dir.as_std_path());
			return Err(Error::custom(format!(
				"Generated symbol for artboard '{}' appears to have no inner content. SVG file: '{}'",
				artboard.name,
				svg_file.path()
			)));
		}

		symbols.push(symbol);
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

/// Finds the SVG file corresponding to an artboard in the cache directory.
/// The file path structure mirrors the artboard name (e.g., "ico/user/fill" -> "ico/user/fill.svg").
fn find_svg_file_for_artboard(cache_dir: &SPath, artboard_name: &str) -> Result<simple_fs::SFile> {
	// sketchtool exports files preserving the artboard name path structure
	// e.g., artboard "ico/user/fill" becomes "cache_dir/ico/user/fill.svg"
	let expected_path = cache_dir.join(format!("{artboard_name}.svg"));

	if expected_path.exists() {
		return simple_fs::SFile::new(expected_path.as_str()).map_err(Error::custom_from_err);
	}

	// If the expected path doesn't exist, fail immediately with a clear error
	// Don't do fuzzy matching as it leads to returning wrong files
	Err(Error::custom(format!(
		"SVG file not found for artboard '{}'. Expected path: '{}' does not exist.",
		artboard_name, expected_path
	)))
}

/// Converts an SVG file content to a symbol element.
fn convert_svg_to_symbol(svg_content: &str, symbol_id: &str) -> Option<String> {
	// Extract viewBox from the SVG
	let viewbox = xmls::extract_root_attribute(svg_content, "viewBox")?;

	// Extract the inner nodes (everything between <svg ...> and </svg>)
	let inner_nodes = xmls::extract_root_inner_nodes(svg_content)?;

	// If no inner nodes, return None to signal an error
	if inner_nodes.is_empty() {
		return None;
	}

	// Canonicalize all id attributes within the inner nodes
	let transformed_nodes = xmls::transform_nodes_id_attributes(inner_nodes, strings::canonicalize_name);

	// Convert nodes back to string
	let inner_content = xmls::nodes_to_string(&transformed_nodes);

	// If inner content is empty after transformation, return None
	if inner_content.trim().is_empty() {
		return None;
	}

	// Indent the inner content for proper formatting
	let indented_content = indent_content(&inner_content, 4);

	// Final check: if indented content is empty, something went wrong
	if indented_content.trim().is_empty() {
		return None;
	}

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

		// If single file output, move the exported file from cache to the target path
		if let Some(ref cache) = cache_dir {
			// sketchtool outputs files in subdirectory structure matching the artboard name path
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
			// For multi-file output, build paths based on artboard names
			// sketchtool exports files with paths matching artboard names (e.g., "ico/user/fill.svg")
			for artboard in artboards {
				let file_path = output_path.join(format!("{}.{format}", artboard.name));
				exported_files.push(file_path.to_string());
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
