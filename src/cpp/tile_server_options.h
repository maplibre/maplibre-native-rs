#include "rust/cxx.h"
#include <memory>

namespace mbgl {
    class TileServerOptions;
}

namespace mln::bridge::tile_server_options {

std::unique_ptr<mbgl::TileServerOptions> new_();
void withBaseUrl(mbgl::TileServerOptions &tile_server_options, rust::Slice<const uint8_t> path);
void withUriSchemeAlias(mbgl::TileServerOptions &tile_server_options, rust::Slice<const uint8_t> path);
void withSourceTemplate(mbgl::TileServerOptions &tile_server_options,
                        rust::Slice<const uint8_t> styleTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    );
void withSpritesTemplate(mbgl::TileServerOptions &tile_server_options,
                        rust::Slice<const uint8_t> spritesTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    );
void withGlyphsTemplate(mbgl::TileServerOptions &tile_server_options,
                        rust::Slice<const uint8_t> glyphsTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    );
void withTileTemplate(mbgl::TileServerOptions &tile_server_options,
                        rust::Slice<const uint8_t> tileTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    );
void withApiKeyParameterName(mbgl::TileServerOptions &tile_server_options,
                        rust::Slice<const uint8_t> apiKeyParameterName
                    );
void setRequiresApiKey(mbgl::TileServerOptions &tile_server_options, bool apiKeyRequired);

}
