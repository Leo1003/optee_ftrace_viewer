use crate::ftrace::{FtraceError, RawFtrace};
use std::{iter::FusedIterator, time::Duration};

#[derive(Clone, Debug)]
pub struct FtraceTree {
    trace_info: String,
    children: Vec<FtraceNode>,
}

#[allow(dead_code)]
impl FtraceTree {
    pub fn new(trace_info: String, children: Vec<FtraceNode>) -> Self {
        Self {
            trace_info,
            children,
        }
    }

    pub fn from_root_node(trace_info: String, root: FtraceNode) -> Self {
        Self {
            trace_info,
            children: root.children,
        }
    }

    pub fn trace_info(&self) -> &str {
        &self.trace_info
    }

    pub fn children(&self) -> impl Iterator<Item = &FtraceNode> {
        self.children.iter()
    }

    pub fn children_mut(&mut self) -> impl Iterator<Item = &mut FtraceNode> {
        self.children.iter_mut()
    }

    pub fn dfs_iter(&self) -> FtraceDfsIter<'_> {
        FtraceDfsIter {
            stack: vec![self.children.iter()],
        }
    }
}

#[derive(Clone, Debug)]
pub struct FtraceNode {
    children: Vec<FtraceNode>,
    depth: u8,
    func: u64,
    symbol: Option<String>,
    time: Option<Duration>,
}

impl FtraceNode {
    pub fn new(depth: u8, func: u64, time: Option<Duration>) -> Self {
        Self {
            children: Vec::new(),
            depth,
            func,
            symbol: None,
            time,
        }
    }

    pub fn with_start(code: RawFtrace) -> Result<Self, FtraceError> {
        if !code.is_start() {
            return Err(FtraceError::InvalidEntry);
        }

        Ok(Self::new(code.depth(), code.data(), None))
    }

    pub fn end_with(&mut self, code: RawFtrace) -> Result<(), FtraceError> {
        if !code.is_end() {
            return Err(FtraceError::InvalidEntry);
        }

        self.time = Some(Duration::from_nanos(code.data()));
        Ok(())
    }

    pub fn add_child(&mut self, child: FtraceNode) {
        self.children.push(child);
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn func(&self) -> u64 {
        self.func
    }

    pub fn symbol(&self) -> Option<&str> {
        self.symbol.as_deref()
    }

    pub fn set_symbol(&mut self, symbol: String) {
        self.symbol = Some(symbol);
    }

    pub fn time(&self) -> Option<Duration> {
        self.time
    }

    pub fn children(&self) -> impl Iterator<Item = &FtraceNode> {
        self.children.iter()
    }

    pub fn children_mut(&mut self) -> impl Iterator<Item = &mut FtraceNode> {
        self.children.iter_mut()
    }
}

pub struct FtraceDfsIter<'a> {
    stack: Vec<std::slice::Iter<'a, FtraceNode>>,
}

impl<'a> Iterator for FtraceDfsIter<'a> {
    type Item = &'a FtraceNode;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If the stack is empty, we're done,
            let top = self.stack.last_mut()?;
            if let Some(item) = top.next() {
                // Push the children iterator onto the stack
                self.stack.push(item.children.iter());
                return Some(item);
            } else {
                // Pop the empty iterator and continue
                self.stack.pop();
            }
        }
    }
}
impl FusedIterator for FtraceDfsIter<'_> {}
