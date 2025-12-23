use clap::{Args, Parser, Subcommand};

// Note: #[command(version)] automatically adds -V/--version support
#[derive(Parser, Debug)]
#[command(version)]
pub struct CliCmd {
	#[command(subcommand)]
	pub command: Option<CliSubCmd>,
}

#[derive(Subcommand, Debug)]
pub enum CliSubCmd {
	#[command(subcommand)]
	Sketch(SketchCommand),
}

// region:    --- Sketch

#[derive(Subcommand, Debug)]
pub enum SketchCommand {
	/// List artboards from a Sketch file
	ListArtboards(ListArtboardsArgs),

	/// Export artboards from a Sketch file
	Export(ExportArgs),
}

#[derive(Args, Debug)]
pub struct ListArtboardsArgs {
	/// Path to the Sketch file
	pub sketch_file: String,

	/// Optional glob patterns to filter artboards by name (can be specified multiple times)
	#[arg(short, long)]
	pub glob: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
	/// Path to the Sketch file
	pub sketch_file: String,

	/// Optional glob patterns to filter artboards by name (can be specified multiple times)
	#[arg(short, long)]
	pub glob: Vec<String>,

	/// Export format(s): svg, png, jpeg, svg-symbols (comma-delimited or multiple flags)
	#[arg(long, value_delimiter = ',')]
	pub format: Vec<String>,

	/// Output directory for exported files
	#[arg(short, long)]
	pub output: String,

	/// Flatten exported file names (e.g., "ico/user/fill" becomes "ico-user-fill")
	#[arg(long)]
	pub flatten: bool,

	/// Keep the raw export cache directory (.cache-raw-export) instead of deleting it
	#[arg(long)]
	pub keep_raw_export: bool,
}

// endregion: --- Sketch
