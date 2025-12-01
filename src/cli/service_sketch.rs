use crate::service::sketch;
use crate::Result;

pub fn exec_list_artboards(sketch_file: &str) -> Result<()> {
	let artboards = sketch::list_artboards(sketch_file)?;

	for artboard in artboards {
		println!("{}: {}", artboard.id, artboard.name);
	}

	Ok(())
}
