use crate::Result;
use serde::Deserialize;
use std::process::Command;

const SKETCHTOOL_PATH: &str =
	"/Applications/Sketch.app/Contents/Resources/sketchtool/bin/sketchtool";

// region:    --- Types

#[derive(Debug)]
pub struct Artboard {
	pub id: String,
	pub name: String,
}

// endregion: --- Types

// region:    --- Sketchtool JSON Response Types

#[derive(Deserialize)]
struct SketchListResponse {
	pages: Vec<SketchPage>,
}

#[derive(Deserialize)]
struct SketchPage {
	artboards: Vec<SketchArtboard>,
}

#[derive(Deserialize)]
struct SketchArtboard {
	id: String,
	name: String,
}

// endregion: --- Sketchtool JSON Response Types

pub fn list_artboards(sketch_file: &str) -> Result<Vec<Artboard>> {
	let output = Command::new(SKETCHTOOL_PATH)
		.args(["--include-symbols=YES", "list", "artboards", sketch_file])
		.output()
		.map_err(|e| format!("Failed to execute sketchtool: {e}"))?;

	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr);
		return Err(format!("sketchtool failed: {stderr}").into());
	}

	let stdout = String::from_utf8_lossy(&output.stdout);
	let response: SketchListResponse =
		serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse sketchtool output: {e}"))?;

	let artboards = response
		.pages
		.into_iter()
		.flat_map(|page| page.artboards)
		.map(|ab| Artboard { id: ab.id, name: ab.name })
		.collect();

	Ok(artboards)
}
