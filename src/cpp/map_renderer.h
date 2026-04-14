#pragma once

#include <cassert>
#include <cstdint>
#include <mbgl/gfx/headless_frontend.hpp>
#include <mbgl/style/image.hpp>
#include <mbgl/style/sources/geojson_source.hpp>
#include <mbgl/style/layers/symbol_layer.hpp>
#include <mbgl/map/map.hpp>
#include <mbgl/map/map_options.hpp>
#include <mbgl/style/style.hpp>
#include <mbgl/style/source.hpp>
#include <mbgl/util/image.hpp>
#include <mbgl/util/run_loop.hpp>
#include <mbgl/util/premultiply.hpp>
#include <mbgl/util/tile_server_options.hpp>
#include <mbgl/util/size.hpp>
#include <mbgl/storage/resource_options.hpp>
#include <mbgl/webgpu/texture2d.hpp>
#include <memory>
#include <vector>
#include <stdexcept>
#include "rust/cxx.h"
#include "rust_log_observer.h"
#include <mbgl/map/map_observer.hpp>
#include "map_observer.h"

namespace mln {
namespace bridge {

struct Texture;
struct TextureView;

constexpr size_t BYTES_PER_PIXEL = 4; // rgba

struct BridgeImage;

class MapRenderer {
public:
    explicit MapRenderer(std::unique_ptr<mbgl::HeadlessFrontend> frontendInstance,
                         std::shared_ptr<MapObserver> mapObserverInstance,
                         std::unique_ptr<mbgl::Map> mapInstance)
        : frontend(std::move(frontendInstance)),
          mapObserverInstance(mapObserverInstance),
          map(std::move(mapInstance)) {}
    ~MapRenderer() {}

    std::shared_ptr<MapObserver> observer() {
        return mapObserverInstance;
    }

    std::shared_ptr<mbgl::webgpu::Texture2D> getTexture() {
        // TODO: don't like the static pointer cast
        auto ptr = std::static_pointer_cast<mbgl::webgpu::Texture2D>(this->frontend->getTexture());
        assert(ptr);
        return ptr;
    }

    void style_add_image(rust::Str id, rust::Slice<const unsigned char> data, mbgl::Size size, bool single_distance_field) {
        mbgl::PremultipliedImage image(size, data.data(), data.size());

        const float pixelRatio = 1.0;
        map->getStyle().addImage(std::make_unique<mbgl::style::Image>(std::string(id), std::move(image), pixelRatio, single_distance_field));
    }

    void style_remove_image(rust::Str id) {
        map->getStyle().removeImage(std::string(id));
    }

    void style_add_geojson_source(std::unique_ptr<mbgl::style::GeoJSONSource> source) {
        map->getStyle().addSource(std::move(source));
    }

    void style_add_symbol_layer(std::unique_ptr<mbgl::style::SymbolLayer> layer) {
        map->getStyle().addLayer(std::move(layer));
    }

    void style_load_from_url(const rust::Str styleUrl) {
        map->getStyle().loadURL((std::string)styleUrl);
    }

    std::unique_ptr<BridgeImage> readStillImage() {
        auto image = frontend->readStillImage();
        auto unpremultipliedImage = mbgl::util::unpremultiply(std::move(image));
        return std::make_unique<BridgeImage>(std::move(unpremultipliedImage.data), unpremultipliedImage.size);
    }

    void render_once() {
        frontend->renderOnce(*map);
    }

    std::unique_ptr<std::string> render() {
        auto result = frontend->render(*map);
        auto unpremultipliedImage = mbgl::util::unpremultiply(std::move(result.image));

        const size_t pixelCount = unpremultipliedImage.size.width * unpremultipliedImage.size.height;
        std::string data;
        data.reserve(pixelCount * BYTES_PER_PIXEL);

        uint32_t width = unpremultipliedImage.size.width;
        uint32_t height = unpremultipliedImage.size.height;
        data.append(reinterpret_cast<const char*>(&width), sizeof(uint32_t));
        data.append(reinterpret_cast<const char*>(&height), sizeof(uint32_t));

        const char* pixelData = reinterpret_cast<const char*>(unpremultipliedImage.data.get());
        data.append(pixelData, pixelCount * BYTES_PER_PIXEL);

        return std::make_unique<std::string>(std::move(data));
    }

    void setSize(const mbgl::Size& size) {
        if (size.width == 0 || size.height == 0)
            return;
        frontend->setSize(size);
        map->setSize(size);
    }

    void setDebugFlags(mbgl::MapDebugOptions debugFlags) {
        map->setDebug(debugFlags);
    }

    void setCamera(double lat, double lon, double zoom, double bearing, double pitch) {
        mbgl::CameraOptions cameraOptions;
        cameraOptions.withCenter(mbgl::LatLng{lat, lon}).withZoom(zoom).withBearing(bearing).withPitch(pitch);
        map->jumpTo(cameraOptions);
    }

    void moveBy(const mbgl::ScreenCoordinate& delta) {
        map->moveBy(delta);
    }

    void scaleBy(double scale, const mbgl::ScreenCoordinate& pos) {
        map->scaleBy(scale, pos);
    }
public:
    mbgl::util::RunLoop runLoop;
    // Due to CXX limitations, make all these public and access them from the regular functions below
    // Hold all objects here, because frontent and the observers are passed by reference to the map
    std::unique_ptr<mbgl::HeadlessFrontend> frontend;
    std::shared_ptr<MapObserver> mapObserverInstance;
    std::unique_ptr<mbgl::Map> map;
};

inline std::unique_ptr<MapRenderer> MapRenderer_new(
            mbgl::MapMode mapMode,
            uint32_t width,
            uint32_t height,
            float pixelRatio,
            const mbgl::ResourceOptions& resourceOptions
) {
    mbgl::Size size = {width, height};
    auto mapObserver = std::make_shared<MapObserver>();
    auto frontend = std::make_unique<mbgl::HeadlessFrontend>(size, pixelRatio);

    mbgl::MapOptions mapOptions;
    mapOptions.withMapMode(mapMode).withSize(size).withPixelRatio(pixelRatio);

    // Set up logging observer for Rust bridge
    auto logObserver = std::make_unique<mln::bridge::RustLogObserver>();
    mbgl::Log::setObserver(std::move(logObserver));
    auto map = std::make_unique<mbgl::Map>(*frontend, *mapObserver, mapOptions, resourceOptions);

    return std::make_unique<MapRenderer>(std::move(frontend), mapObserver, std::move(map));
}

struct BridgeImage {
    public:
        BridgeImage(std::unique_ptr<uint8_t[]> data, mbgl::Size size): mSize(size), mData(std::move(data)) {}

        const uint8_t* get() const {
            return mData.get();
        }

        size_t bufferLength() const {
            const size_t pixelCount = mSize.width * mSize.height;
            return pixelCount * BYTES_PER_PIXEL;
        }

        mbgl::Size size() const {
            return mSize;
        }

    private:
        mbgl::Size mSize;
        std::unique_ptr<uint8_t[]> mData;
};

} // namespace bridge
} // namespace mln
