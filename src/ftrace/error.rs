use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq, Hash)]
pub enum FtraceError {
    #[error("Invalid ftrace entry")]
    InvalidEntry,
}
