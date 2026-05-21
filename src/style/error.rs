/// Error returned when MapLibre Native rejects a style mutation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum StyleError {
    /// MapLibre Native rejected or failed to apply the style mutation.
    #[error("style error: {0}")]
    Native(String),
}

impl From<cxx::Exception> for StyleError {
    fn from(value: cxx::Exception) -> Self {
        Self::Native(value.to_string())
    }
}
