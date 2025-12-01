use crate::Result;
use crate::service::sketch::Artboard;
use serde::Deserialize;
use simple_fs::SPath;
use std::collections::HashMap;
use std::process::Command;

const SKETCHTOOL_PATH: &str = "/Applications/Sketch.app/Contents/Resources/sketchtool/bin/sketchtool";

// region:    --- Sketchtool JSON Response Types

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SketchMetadataResponse {
	pages_and_artboards: HashMap<String, SketchPage>,
}

#[derive(Deserialize)]
struct SketchPage {
	artboards: HashMap<String, SketchArtboard>,
}

#[derive(Deserialize)]
struct SketchArtboard {
	name: String,
}

// endregion: --- Sketchtool JSON Response Types

pub fn list_artboards(sketch_file: impl AsRef<SPath>) -> Result<Vec<Artboard>> {
	let sketch_file = sketch_file.as_ref();

	let output = Command::new(SKETCHTOOL_PATH)
		.args(["metadata", sketch_file.as_str()])
		.output()
		.map_err(|e| format!("Failed to execute sketchtool: {e}"))?;

	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr);
		return Err(format!("sketchtool failed: {stderr}").into());
	}

	let stdout = String::from_utf8_lossy(&output.stdout);
	let response: SketchMetadataResponse =
		serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse sketchtool output: {e}"))?;

	let artboards = response
		.pages_and_artboards
		.into_values()
		.flat_map(|page| page.artboards)
		.map(|(uid, ab)| Artboard { uid, name: ab.name })
		.collect();

	Ok(artboards)
}
