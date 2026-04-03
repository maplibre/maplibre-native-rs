#include "tile_server_options.h"
#include "mbgl/util/tile_server_options.hpp"
#include "rust/cxx.h"
#include "util.h"
#include <memory>
#include <optional>

namespace mln::bridge::tile_server_options {

std::unique_ptr<mbgl::TileServerOptions> new_() {
    auto ptr = std::make_unique<mbgl::TileServerOptions>();
    *ptr = mbgl::TileServerOptions::DefaultConfiguration();
    return ptr;
}
void withBaseUrl(mbgl::TileServerOptions &tile_server_options, rust::Slice<const uint8_t> path) {
    tile_server_options.withBaseURL(rustSliceToString(path));
}

void withUriSchemeAlias(mbgl::TileServerOptions &tile_server_options, rust::Slice<const uint8_t> alias) {
    tile_server_options.withUriSchemeAlias(rustSliceToString(alias));
}
void withSourceTemplate(mbgl::TileServerOptions &tile_server_options, 
                        rust::Slice<const uint8_t> styleTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    ) {
    tile_server_options.withSourceTemplate(rustSliceToString(styleTemplate),
                                        rustSliceToString(domainName), 
                                        std::optional<std::string>{rustSliceToString(versionPrefix)}
                                    );
}

void withSpritesTemplate(mbgl::TileServerOptions &tile_server_options, 
                        rust::Slice<const uint8_t> spritesTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    ) {
    tile_server_options.withSpritesTemplate(rustSliceToString(spritesTemplate),
                                        rustSliceToString(domainName), 
                                        std::optional<std::string>{rustSliceToString(versionPrefix)}
                                    );

}

void withGlyphsTemplate(mbgl::TileServerOptions &tile_server_options, 
                        rust::Slice<const uint8_t> glyphsTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    ) {
    tile_server_options.withGlyphsTemplate(rustSliceToString(glyphsTemplate),
                                        rustSliceToString(domainName), 
                                        std::optional<std::string>{rustSliceToString(versionPrefix)}
                                    );                   
}
void withTileTemplate(mbgl::TileServerOptions &tile_server_options, 
                        rust::Slice<const uint8_t> tileTemplate,
                        rust::Slice<const uint8_t> domainName,
                        rust::Slice<const uint8_t> versionPrefix
                    ) {
    tile_server_options.withGlyphsTemplate(rustSliceToString(tileTemplate),
                                        rustSliceToString(domainName), 
                                        std::optional<std::string>{rustSliceToString(versionPrefix)}
                                    );
}

void withApiKeyParameterName(mbgl::TileServerOptions &tile_server_options,
                        rust::Slice<const uint8_t> apiKeyParameterName) {
    tile_server_options.withApiKeyParameterName(rustSliceToString(apiKeyParameterName));
}

void setRequiresApiKey(mbgl::TileServerOptions &tile_server_options, bool apiKeyRequired) {
    tile_server_options.setRequiresApiKey(apiKeyRequired);
}

} // namespace