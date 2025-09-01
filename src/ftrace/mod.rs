mod error;
mod file;
mod raw_entry;
mod symbol;
mod tree;

pub use error::FtraceError;
pub use file::FtraceFile;
pub use raw_entry::RawFtrace;
pub use symbol::{ElfInfo, RegionData, RegionFlags, SymbolInfo};
pub use tree::{FtraceNode, FtraceTree};

pub const MAGIC: &[u8] = b"FTRACE\x00\x01";
