#pragma once

#include "rust/cxx.h"
#include <memory>

namespace mln {
    class TileServerOptions;
}

namespace mln::bridge::tile_server_options {

std::unique_ptr<mln::TileServerOptions> new_();
void withBaseUrl(mln::TileServerOptions &tile_server_options, rust::Slice<const uint8_t> path);
void withUriSchemeAlias(mln::TileServerOptions &tile_server_options, rust::Slice<const uint8_t> path);
void withSourceTemplate(mln::TileServerOptions &tile_server_options,
                        rust::Slice<const uint8_t> styleTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    );
void withSpritesTemplate(mln::TileServerOptions &tile_server_options,
                        rust::Slice<const uint8_t> spritesTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    );
void withGlyphsTemplate(mln::TileServerOptions &tile_server_options,
                        rust::Slice<const uint8_t> glyphsTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    );
void withTileTemplate(mln::TileServerOptions &tile_server_options,
                        rust::Slice<const uint8_t> tileTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    );
void withApiKeyParameterName(mln::TileServerOptions &tile_server_options,
                        rust::Slice<const uint8_t> apiKeyParameterName
                    );
void setRequiresApiKey(mln::TileServerOptions &tile_server_options, bool apiKeyRequired);

}
