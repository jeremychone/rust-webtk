use crate::service::sketch;
use crate::Result;
use simple_fs::SPath;

pub fn exec_list_artboards(sketch_file: &str) -> Result<()> {
	let sketch_file = SPath::new(sketch_file);
	let artboards = sketch::list_artboards(&sketch_file)?;

	for artboard in artboards {
		println!("{}: {}", artboard.uid, artboard.name);
	}

	Ok(())
}
