use clap::Parser;
use std::path::PathBuf;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    pub ftrace_path: PathBuf,

    #[arg(short, long)]
    pub elf: Vec<PathBuf>,
}
