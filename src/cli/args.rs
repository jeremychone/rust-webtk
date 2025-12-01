use clap::{Parser, Subcommand};

// Note: #[command(version)] automatically adds -V/--version support
#[derive(Parser, Debug)]
#[command(version)]
pub struct CliArgs {
	#[command(subcommand)]
	pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
	/// Sketch-related commands
	Sketch {
		#[command(subcommand)]
		command: SketchCommand,
	},
}

#[derive(Subcommand, Debug)]
pub enum SketchCommand {
	/// List artboards from a Sketch file
	ListArtboards {
		/// Path to the Sketch file
		sketch_file: String,

		/// Optional glob patterns to filter artboards by name (can be specified multiple times)
		#[arg(short, long)]
		glob: Vec<String>,
	},
}
