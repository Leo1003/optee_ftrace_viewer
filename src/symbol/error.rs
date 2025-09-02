use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq, Hash)]
pub enum SymbolError {
    #[error("Uuid error: {0}")]
    UuidError(#[from] uuid::Error),
    #[error("Failed to parse symbol information")]
    InvalidSymbolInfo,
    #[error("Failed to parse region table")]
    InvalidRegionTable,
    #[error("Failed to parse region flags")]
    InvalidRegionFlags,
}
