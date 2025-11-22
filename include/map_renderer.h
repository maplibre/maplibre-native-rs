#pragma once

#include <mbgl/gfx/headless_frontend.hpp>
#include <mbgl/map/map.hpp>
#include <mbgl/map/map_options.hpp>
#include <mbgl/style/style.hpp>
#include <mbgl/util/image.hpp>
#include <mbgl/util/run_loop.hpp>
#include <mbgl/util/premultiply.hpp>
#include <mbgl/util/tile_server_options.hpp>
#include <memory>
#include <vector>
#include <stdexcept>
#include "rust/cxx.h"
#include "rust_log_observer.h"
#include <mbgl/map/map_observer.hpp>
#include <mbgl/renderer/renderer_observer.hpp>

namespace mln {
namespace bridge {

using namespace mbgl; // TODO: why is this required

class MapRenderer {
public:
    explicit MapRenderer(std::unique_ptr<mbgl::HeadlessFrontend> frontendInstance,
                         std::unique_ptr<mbgl::RendererObserver> observerInstance,
                         std::unique_ptr<mbgl::Map> mapInstance)
        : frontend(std::move(frontendInstance)),
          observer(std::move(observerInstance)),
          map(std::move(mapInstance)) {}
    ~MapRenderer() {}

public:
    mbgl::util::RunLoop runLoop;
    // Due to CXX limitations, make all these public and access them from the regular functions below
    std::unique_ptr<mbgl::HeadlessFrontend> frontend;
    std::unique_ptr<mbgl::RendererObserver> observer;
    std::unique_ptr<mbgl::Map> map;
};

inline std::unique_ptr<MapRenderer> MapRenderer_new_with_observer(
            mbgl::MapMode mapMode,
            uint32_t width,
            uint32_t height,
            float pixelRatio,
            rust::Slice<const uint8_t> cachePath,
            rust::Slice<const uint8_t> assetRoot,
            const rust::Str apiKey,
            const rust::Str baseUrl,
            const rust::Str uriSchemeAlias,
            const rust::Str apiKeyParameterName,
            const rust::Str sourceTemplate,
            const rust::Str styleTemplate,
            const rust::Str spritesTemplate,
            const rust::Str glyphsTemplate,
            const rust::Str tileTemplate,
            bool requiresApiKey,
            std::unique_ptr<mbgl::RendererObserver> observer
) {

    mbgl::Size size = {width, height};

    auto frontend = std::make_unique<mbgl::HeadlessFrontend>(size, pixelRatio);
    frontend->setObserver(*observer);

    mbgl::TileServerOptions options = mbgl::TileServerOptions()
        .withBaseURL((std::string)baseUrl)
        .withUriSchemeAlias((std::string)uriSchemeAlias)
        .withSourceTemplate((std::string)sourceTemplate, "", {})
        .withStyleTemplate((std::string)styleTemplate, "maps", {})
        .withSpritesTemplate((std::string)spritesTemplate, "", {})
        .withGlyphsTemplate((std::string)glyphsTemplate, "fonts", {})
        .withTileTemplate((std::string)tileTemplate, "tiles", {})
        .withApiKeyParameterName((std::string)apiKeyParameterName)
        .setRequiresApiKey(requiresApiKey);

    mbgl::ResourceOptions resourceOptions;
    resourceOptions
        .withCachePath(std::string(reinterpret_cast<const char*>(cachePath.data()), cachePath.size()))
        .withAssetPath(std::string(reinterpret_cast<const char*>(assetRoot.data()), assetRoot.size()))
        .withApiKey((std::string)apiKey)
        .withTileServerOptions(options);

    mbgl::MapOptions mapOptions;
    mapOptions.withMapMode(mapMode).withSize(size).withPixelRatio(pixelRatio);

    // Set up logging observer for Rust bridge
    auto logObserver = std::make_unique<mln::bridge::RustLogObserver>();
    mbgl::Log::setObserver(std::move(logObserver));

    auto map = std::make_unique<mbgl::Map>(*frontend, mbgl::MapObserver::nullObserver(), mapOptions, resourceOptions);

    return std::make_unique<MapRenderer>(std::move(frontend), std::move(observer), std::move(map));
}

inline std::unique_ptr<MapRenderer> MapRenderer_new(
            mbgl::MapMode mapMode,
            uint32_t width,
            uint32_t height,
            float pixelRatio,
            rust::Slice<const uint8_t> cachePath,
            rust::Slice<const uint8_t> assetRoot,
            const rust::Str apiKey,
            const rust::Str baseUrl,
            const rust::Str uriSchemeAlias,
            const rust::Str apiKeyParameterName,
            const rust::Str sourceTemplate,
            const rust::Str styleTemplate,
            const rust::Str spritesTemplate,
            const rust::Str glyphsTemplate,
            const rust::Str tileTemplate,
            bool requiresApiKey

) {
    return MapRenderer_new_with_observer(
        mapMode, 
        width, 
        height, 
        pixelRatio, 
        cachePath, 
        assetRoot, 
        apiKey, 
        baseUrl, 
        uriSchemeAlias, 
        apiKeyParameterName, 
        sourceTemplate, 
        styleTemplate, 
        spritesTemplate, 
        glyphsTemplate, 
        tileTemplate, 
        requiresApiKey,
        nullptr);
}

inline std::unique_ptr<std::string> MapRenderer_render(MapRenderer& self) {
    auto result = self.frontend->render(*self.map);
    auto unpremultipliedImage = mbgl::util::unpremultiply(std::move(result.image));

    // Prepare string with dimensions and pixel data
    const size_t pixelCount = unpremultipliedImage.size.width * unpremultipliedImage.size.height;
    std::string data;
    data.reserve(sizeof(uint32_t) * 2 + pixelCount * 4);

    // First 8 bytes: width and height as uint32_t (little-endian)
    uint32_t width = unpremultipliedImage.size.width;
    uint32_t height = unpremultipliedImage.size.height;
    data.append(reinterpret_cast<const char*>(&width), sizeof(uint32_t));
    data.append(reinterpret_cast<const char*>(&height), sizeof(uint32_t));

    // Append the unpremultiplied RGBA pixel data directly
    const char* pixelData = reinterpret_cast<const char*>(unpremultipliedImage.data.get());
    data.append(pixelData, pixelCount * 4);

    return std::make_unique<std::string>(std::move(data));
}

inline void MapRenderer_setDebugFlags(MapRenderer& self, mbgl::MapDebugOptions debugFlags) {
    self.map->setDebug(debugFlags);
}

inline void MapRenderer_setCamera(
    MapRenderer& self, double lat, double lon, double zoom, double bearing, double pitch) {
    // TODO: decide if this is the right approach,
    //       or if we want to cache camera options in the instance,
    //       and have several setters for each property.
    mbgl::CameraOptions cameraOptions;
    cameraOptions.withCenter(mbgl::LatLng{lat, lon}).withZoom(zoom).withBearing(bearing).withPitch(pitch);
    self.map->jumpTo(cameraOptions);
}

inline void MapRenderer_getStyle_loadURL(MapRenderer& self, const rust::Str styleUrl) {
    self.map->getStyle().loadURL((std::string)styleUrl);
}

} // namespace bridge
} // namespace mln
