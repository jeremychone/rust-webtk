use clap::Parser;

// Note: #[command(version)] automatically adds -V/--version support
#[derive(Parser, Debug)]
#[command(version)]
pub struct CliArgs {}
