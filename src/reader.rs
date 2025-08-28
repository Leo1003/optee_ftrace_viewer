use crate::ftrace::FtraceFile;
use color_eyre::eyre::{Result, eyre};
use std::path::Path;

pub use crate::ftrace::{FtraceNode, FtraceTree};

pub async fn build_ftrace_tree_from_file(path: &Path) -> Result<FtraceTree> {
    let mut file = FtraceFile::open(path).await?;

    let mut root = FtraceNode::new(0, 0, None);
    root = recursive_build_tree(&mut file, root, 0).await?;

    Ok(FtraceTree::from_root_node(
        file.trace_info().to_owned(),
        root,
    ))
}

async fn recursive_build_tree(
    file: &mut FtraceFile,
    mut cur_node: FtraceNode,
    depth: u8,
) -> Result<FtraceNode> {
    if cur_node.depth() > depth {
        // Save the node
        let next_node = cur_node;
        // Replace current node with a dummy node to hold children
        cur_node = FtraceNode::new(depth, 0, None);
        let child = Box::pin(recursive_build_tree(file, next_node, depth + 1)).await?;
        cur_node.add_child(child);
    } else if cur_node.depth() < depth {
        return Err(eyre!(
            "Unexpected depth: current node depth {}, expected {}",
            cur_node.depth(),
            depth
        ));
    }

    while let Some(entry) = file.next_entry().await? {
        if entry.is_start() {
            let mut child = FtraceNode::with_start(entry)?;
            child = Box::pin(recursive_build_tree(file, child, depth + 1)).await?;
            cur_node.add_child(child);
        } else if entry.is_end() {
            cur_node.end_with(entry)?;
            break;
        }
    }

    Ok(cur_node)
}
