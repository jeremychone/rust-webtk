use crate::Result;
use crate::cli::args::SketchCommand;
use crate::service::sketch;
use simple_fs::SPath;

pub fn exec_command(command: SketchCommand) -> Result<()> {
	match command {
		SketchCommand::ListArtboards { sketch_file, glob } => exec_list_artboards(&sketch_file, glob),
		SketchCommand::Export {
			sketch_file,
			glob,
			format,
			output,
			flatten,
			keep_raw_export,
		} => exec_export(&sketch_file, glob, format, &output, flatten, keep_raw_export),
	}
}

fn exec_list_artboards(sketch_file: &str, globs: Vec<String>) -> Result<()> {
	let sketch_file = SPath::new(sketch_file);
	let glob_refs: Vec<&str> = globs.iter().map(|s| s.as_str()).collect();
	let glob_arg = if glob_refs.is_empty() {
		None
	} else {
		Some(glob_refs.as_slice())
	};
	let artboards = sketch::list_artboards(&sketch_file, glob_arg)?;

	for artboard in artboards {
		println!("{}: {}", artboard.uid, artboard.name);
	}

	Ok(())
}

fn exec_export(
	sketch_file: &str,
	globs: Vec<String>,
	formats: Vec<String>,
	output: &str,
	flatten: bool,
	keep_raw_export: bool,
) -> Result<()> {
	let sketch_file = SPath::new(sketch_file);
	let output_dir = SPath::new(output);

	let glob_refs: Vec<&str> = globs.iter().map(|s| s.as_str()).collect();
	let glob_arg = if glob_refs.is_empty() {
		None
	} else {
		Some(glob_refs.as_slice())
	};

	let format_refs: Vec<&str> = formats.iter().map(|s| s.as_str()).collect();

	let exported = sketch::export_artboards(
		&sketch_file,
		glob_arg,
		&format_refs,
		&output_dir,
		flatten,
		keep_raw_export,
	)?;

	for path in exported {
		println!("Exported: {path}");
	}

	Ok(())
}
