/// A stable image ID handle that can be used after an image object is moved.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageId(String);

impl ImageId {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    #[must_use]
    /// Returns the image ID as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ImageId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
