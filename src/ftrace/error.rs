use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FtraceError;

impl Display for FtraceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FtraceError")
    }
}

impl std::error::Error for FtraceError {}
