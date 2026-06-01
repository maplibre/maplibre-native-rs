/// Error returned by style operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StyleError {
    /// MapLibre Native rejected the operation.
    #[error("style error: {0}")]
    Native(String),

    /// The supplied JSON string could not be parsed.
    #[cfg(feature = "json")]
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// A JSON number could not be represented as a finite `f64`.
    #[cfg(feature = "json")]
    #[error("invalid JSON number: {0}")]
    JsonNumber(String),
}

impl From<cxx::Exception> for StyleError {
    fn from(value: cxx::Exception) -> Self {
        Self::Native(value.to_string())
    }
}
