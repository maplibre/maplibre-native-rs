use crate::renderer::bridge::tile_server_options;
use cxx::UniquePtr;
use std::path::PathBuf;

pub struct TileServerOptions {
    ptr: UniquePtr<tile_server_options::TileServerOptions>,
}

impl TileServerOptions {
    #[must_use]
    pub fn new() -> Self {
        Self {
            ptr: tile_server_options::new_tile_server_options(),
        }
    }

    #[must_use]
    pub fn with_base_url(mut self, path: PathBuf) {
        tile_server_options::withBaseUrl(
            self.ptr.pin_mut(),
            path.into_os_string().into_encoded_bytes().as_slice(),
        );
    }

    #[must_use]
    pub fn with_uri_scheme_alias(mut self, path: PathBuf) -> Self {
        tile_server_options::withUriSchemeAlias(
            self.ptr.pin_mut(),
            path.into_os_string().into_encoded_bytes().as_slice(),
        );
        self
    }

    #[must_use]
    pub fn with_source_template(
        mut self,
        style_template: PathBuf,
        domain_name: &str,
        version_prefix: &str,
    ) -> Self {
        tile_server_options::withSourceTemplate(
            self.ptr.pin_mut(),
            style_template
                .into_os_string()
                .into_encoded_bytes()
                .as_slice(),
            domain_name.as_bytes(),
            version_prefix.as_bytes(),
        );
        self
    }

    #[must_use]
    pub fn with_sprites_template(
        mut self,
        sprites_template: PathBuf,
        domain_name: &str,
        version_prefix: &str,
    ) -> Self {
        tile_server_options::withSpritesTemplate(
            self.ptr.pin_mut(),
            sprites_template
                .into_os_string()
                .into_encoded_bytes()
                .as_slice(),
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
            glyphs_template
                .into_os_string()
                .into_encoded_bytes()
                .as_slice(),
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
            tile_template
                .into_os_string()
                .into_encoded_bytes()
                .as_slice(),
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

    #[must_use]
    pub(crate) fn into_ptr(self) -> UniquePtr<tile_server_options::TileServerOptions> {
        self.ptr
    }
}

// /// Sets tile server base URL
// ///
// /// Default: <https://demotiles.maplibre.org>
// #[must_use]
// #[allow(clippy::needless_pass_by_value, reason = "false positive")]
// pub fn with_base_url(mut self, base_url: url::Url) -> Self {
//     self.base_url = base_url;
//     self
// }

// /// Sets custom URI scheme alias
// ///
// /// Default: "maplibre"
// #[must_use]
// #[allow(clippy::needless_pass_by_value, reason = "false positive")]
// pub fn with_uri_scheme_alias(mut self, uri_scheme_alias: impl ToString) -> Self {
//     self.uri_scheme_alias = uri_scheme_alias.to_string();
//     self
// }

// /// Sets source JSON URL template
// ///
// /// Default: "/tiles/{domain}.json"
// #[must_use]
// #[allow(clippy::needless_pass_by_value, reason = "false positive")]
// pub fn with_source_template(mut self, source_template: impl ToString) -> Self {
//     self.source_template = source_template.to_string();
//     self
// }
// /// Sets style JSON URL template
// ///
// /// Default: "{path}.json"
// #[must_use]
// #[allow(clippy::needless_pass_by_value, reason = "false positive")]
// pub fn with_style_template(mut self, style_template: impl ToString) -> Self {
//     self.style_template = style_template.to_string();
//     self
// }

// /// Sets sprite URL template
// ///
// /// Default: "/{path}/sprite{scale}.{format}"
// #[must_use]
// #[allow(clippy::needless_pass_by_value, reason = "false positive")]
// pub fn with_sprites_template(mut self, sprites_template: impl ToString) -> Self {
//     self.sprites_template = sprites_template.to_string();
//     self
// }
