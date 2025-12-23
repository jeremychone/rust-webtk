use crate::Result;
use crate::cli::cmd::{CliCmd, CliSubCmd};
use crate::cli::exec_sketch;
use clap::Parser as _;

pub fn execute() -> Result<()> {
	let cli_cmd = CliCmd::parse();

	let Some(sub_cmd) = cli_cmd.command else {
		println!("Hello webtk world! Use --help for available commands.");
		return Ok(());
	};

	let res: Result<()> = match sub_cmd {
		CliSubCmd::Sketch(command) => exec_sketch::exec_command(command),
	};

	res?;

	Ok(())
}
