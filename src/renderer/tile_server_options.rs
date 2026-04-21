//! Tile server options builder.

use std::fmt;
use std::path::PathBuf;

use cxx::UniquePtr;

use crate::renderer::bridge::tile_server_options;

/// Configuration options for a tile server.
pub struct TileServerOptions {
    ptr: UniquePtr<tile_server_options::TileServerOptions>,
}

impl Default for TileServerOptions {
    /// Create new tile server options object
    fn default() -> Self {
        let ptr = tile_server_options::new_tile_server_options();
        assert!(!ptr.is_null(), "nullptr to TileServerOptions received");
        Self { ptr }
    }
}

impl fmt::Debug for TileServerOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TileServerOptions").field("Pointer", &self.ptr.as_ptr()).finish()
    }
}

impl TileServerOptions {
    /// Set base url
    #[must_use]
    pub fn with_base_url(mut self, path: PathBuf) -> Self {
        tile_server_options::withBaseUrl(
            self.ptr.pin_mut(),
            path.into_os_string().into_encoded_bytes().as_slice(),
        );
        self
    }

    /// Set uri scheme alias
    #[must_use]
    pub fn with_uri_scheme_alias(mut self, path: PathBuf) -> Self {
        tile_server_options::withUriSchemeAlias(
            self.ptr.pin_mut(),
            path.into_os_string().into_encoded_bytes().as_slice(),
        );
        self
    }

    /// add source template
    #[must_use]
    pub fn with_source_template(
        mut self,
        style_template: PathBuf,
        domain_name: &str,
        version_prefix: &str,
    ) -> Self {
        tile_server_options::withSourceTemplate(
            self.ptr.pin_mut(),
            style_template.into_os_string().into_encoded_bytes().as_slice(),
            domain_name.as_bytes(),
            version_prefix.as_bytes(),
        );
        self
    }

    /// Add sprites template
    #[must_use]
    pub fn with_sprites_template(
        mut self,
        sprites_template: PathBuf,
        domain_name: &str,
        version_prefix: &str,
    ) -> Self {
        tile_server_options::withSpritesTemplate(
            self.ptr.pin_mut(),
            sprites_template.into_os_string().into_encoded_bytes().as_slice(),
            domain_name.as_bytes(),
            version_prefix.as_bytes(),
        );
        self
    }

    /// Sets glyph URL template
    #[must_use]
    pub fn with_glyphs_template(
        mut self,
        glyphs_template: PathBuf,
        domain_name: &str,
        version_prefix: &str,
    ) -> Self {
        tile_server_options::withGlyphsTemplate(
            self.ptr.pin_mut(),
            glyphs_template.into_os_string().into_encoded_bytes().as_slice(),
            domain_name.as_bytes(),
            version_prefix.as_bytes(),
        );
        self
    }

    /// Sets tile URL template
    #[must_use]
    pub fn with_tile_template(
        mut self,
        tile_template: PathBuf,
        domain_name: &str,
        version_prefix: &str,
    ) -> Self {
        tile_server_options::withTileTemplate(
            self.ptr.pin_mut(),
            tile_template.into_os_string().into_encoded_bytes().as_slice(),
            domain_name.as_bytes(),
            version_prefix.as_bytes(),
        );
        self
    }

    /// Sets API key parameter name
    #[must_use]
    pub fn with_api_key_parameter_name(mut self, api_key_parameter_name: &str) -> Self {
        tile_server_options::withApiKeyParameterName(
            self.ptr.pin_mut(),
            api_key_parameter_name.as_bytes(),
        );
        self
    }

    /// Sets whether API key is required
    #[must_use]
    pub fn set_requires_api_key(mut self, api_key_required: bool) -> Self {
        tile_server_options::setRequiresApiKey(self.ptr.pin_mut(), api_key_required);
        self
    }

    /// Get nonmutable reference to the object
    #[must_use]
    pub(crate) fn as_ref(&self) -> &tile_server_options::TileServerOptions {
        self.ptr.as_ref().unwrap()
    }
}
