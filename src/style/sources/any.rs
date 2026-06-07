use std::fmt;

use cxx::UniquePtr;

use crate::bridge::ffi;
#[cfg(feature = "json")]
use crate::bridge::style_value;
#[cfg(feature = "json")]
use crate::style::{value::build_style_value, StyleError};

/// A style source of a type that does not (yet) have a typed Rust wrapper.
pub struct OpaqueSource {
    source_id: String,
    source: UniquePtr<ffi::CxxSource>,
}

impl OpaqueSource {
    /// Returns the source's ID.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub(crate) fn into_inner(self) -> UniquePtr<ffi::CxxSource> {
        self.source
    }
}

impl fmt::Debug for OpaqueSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpaqueSource").field("source_id", &self.source_id).finish_non_exhaustive()
    }
}

/// A style source of any type, parsed from a style-spec source object.
#[derive(Debug)]
#[non_exhaustive]
pub enum AnySource {
    /// A source of a type that does not have a typed wrapper in this crate yet.
    Opaque(OpaqueSource),
}

impl AnySource {
    // Currently only reachable from the JSON parsing path; gate it so non-`json`
    // builds don't see it as dead code.
    #[cfg(feature = "json")]
    pub(crate) fn from_source_ptr(
        source_id: String,
        source: UniquePtr<ffi::CxxSource>,
    ) -> Option<Self> {
        if source.is_null() {
            return None;
        }

        Some(Self::Opaque(OpaqueSource { source_id, source }))
    }

    /// Parses a single style-spec source object from a JSON string.
    ///
    /// Source objects do not contain their ID, so `id` is provided separately.
    ///
    /// # Errors
    ///
    /// Returns [`StyleError::Json`] if the input is not valid JSON, or
    /// [`StyleError::Native`] if MapLibre Native rejects the source.
    #[cfg(feature = "json")]
    pub fn from_json_str(id: impl AsRef<str>, json: &str) -> Result<Self, StyleError> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        Self::from_json_value(id, &value)
    }

    /// Parses a style-spec source object from a [`serde_json::Value`].
    ///
    /// Source objects do not contain their ID, so `id` is provided separately.
    ///
    /// # Errors
    ///
    /// Returns [`StyleError::JsonNumber`] if a JSON number cannot be converted,
    /// or [`StyleError::Native`] if MapLibre Native rejects the value.
    #[cfg(feature = "json")]
    pub fn from_json_value(
        id: impl AsRef<str>,
        value: &serde_json::Value,
    ) -> Result<Self, StyleError> {
        let id = id.as_ref();
        let style_value = build_style_value(value)?;
        let mut error_message = String::new();
        let source = style_value::source_from_value(id, &style_value, &mut error_message);
        if source.is_null() {
            return Err(StyleError::Native(error_message));
        }
        Self::from_source_ptr(id.to_owned(), source).ok_or(StyleError::Native(error_message))
    }

    /// Returns the source's ID.
    #[must_use]
    pub fn source_id(&self) -> &str {
        match self {
            Self::Opaque(s) => s.source_id(),
        }
    }

    pub(crate) fn into_inner(self) -> UniquePtr<ffi::CxxSource> {
        match self {
            Self::Opaque(s) => s.into_inner(),
        }
    }
}
