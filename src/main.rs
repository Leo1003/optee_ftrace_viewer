use clap::Parser as _;
use color_eyre::eyre::Result;

mod cli;
mod ftrace;
mod reader;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = cli::Cli::parse();

    let tree = reader::build_ftrace_tree_from_file(&args.ftrace_path).await?;
    for node in tree.dfs_iter() {
        println!(
            "{}0x{:016x}(): {:?}",
            "  ".repeat(node.depth() as usize),
            node.func(),
            node.time(),
        );
    }

    Ok(())
}
