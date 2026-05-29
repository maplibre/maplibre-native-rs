/// Stable layer ID handle that can be used after a layer object is moved.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LayerId(String);

impl LayerId {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    #[must_use]
    /// Returns the layer ID as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for LayerId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
