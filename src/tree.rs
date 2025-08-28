use crate::ftrace::{MAGIC, RawFtrace};
use color_eyre::eyre::{Result, eyre};
use memchr::memmem::Finder;
use std::path::Path;
use tokio::{
    fs::File,
    io::{AsyncBufRead, AsyncBufReadExt as _, AsyncReadExt as _, BufReader},
};

#[allow(clippy::module_inception)]
mod tree;

#[allow(unused)]
pub use tree::{FtraceNode, FtraceTree, FtraceTreeError};

pub async fn build_ftrace_tree_from_file(path: &Path) -> Result<FtraceTree> {
    let f = File::open(path).await?;
    let mut reader = BufReader::new(f);
    let mut trace_info = String::new();
    read_to_ftrace_magic(&mut reader, &mut trace_info).await?;

    let mut buf = [0u8; 8];
    let mut root = FtraceNode::new(0, 0, None);
    root = recursive_build_tree(&mut reader, &mut buf, root, 0).await?;

    Ok(FtraceTree::from_root_node(trace_info, root))
}

async fn recursive_build_tree<R>(
    reader: &mut R,
    buf: &mut [u8; 8],
    mut cur_node: FtraceNode,
    depth: u8,
) -> Result<FtraceNode>
where
    R: AsyncBufRead + Unpin,
{
    if cur_node.depth() > depth {
        // Save the node
        let next_node = cur_node;
        // Replace current node with a dummy node to hold children
        cur_node = FtraceNode::new(depth, 0, None);
        let child = Box::pin(recursive_build_tree(reader, buf, next_node, depth + 1)).await?;
        cur_node.add_child(child);
    } else if cur_node.depth() < depth {
        return Err(eyre!(
            "Unexpected depth: current node depth {}, expected {}",
            cur_node.depth(),
            depth
        ));
    }

    while let Some(entry) = read_ftrace_entry(reader, buf).await? {
        if entry.is_start() {
            let mut child = FtraceNode::with_start(entry)?;
            child = Box::pin(recursive_build_tree(reader, buf, child, depth + 1)).await?;
            cur_node.add_child(child);
        } else if entry.is_end() {
            cur_node.end_with(entry)?;
            break;
        }
    }

    Ok(cur_node)
}

async fn read_ftrace_entry<R>(reader: &mut R, buf: &mut [u8; 8]) -> Result<Option<RawFtrace>>
where
    R: AsyncBufRead + Unpin,
{
    match reader.read_exact(buf).await {
        Ok(_) => {}
        Err(e) => {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(None);
            } else {
                return Err(e.into());
            }
        }
    };
    Ok(Some(RawFtrace::from(u64::from_le_bytes(*buf))))
}

async fn read_to_ftrace_magic<R>(reader: &mut R, trace_info: &mut String) -> Result<()>
where
    R: AsyncBufRead + Unpin,
{
    let mut info_buf = Vec::new();
    let finder = Finder::new(MAGIC);
    loop {
        let buf = reader.fill_buf().await?;
        if buf.len() < MAGIC.len() {
            break;
        }

        if let Some(i) = finder.find(buf) {
            info_buf.extend_from_slice(&buf[..i]);
            reader.consume(i + MAGIC.len());

            *trace_info = String::from_utf8(info_buf)?;
            return Ok(());
        }

        let len = buf.len() - MAGIC.len() + 1;
        info_buf.extend_from_slice(&buf[..len]);
        reader.consume(len);
    }

    Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "ftrace magic not found").into())
}
