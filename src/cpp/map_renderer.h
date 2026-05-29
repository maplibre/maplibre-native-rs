#pragma once

#include <mbgl/actor/scheduler.hpp>
#include <mbgl/gfx/headless_frontend.hpp>
#include <mbgl/gfx/renderer_backend.hpp>
#include <mbgl/style/image.hpp>
#include <mbgl/style/layer.hpp>
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

#if defined(MLN_WEBGPU_IMPL_FFI)
#include <mbgl/webgpu/texture2d.hpp>
#include <mbgl/webgpu/renderer_backend.hpp>
#include <mbgl/webgpu/headless_backend.hpp>
#endif

#include <cstdint>
#include <cassert>
#include <memory>
#include <optional>
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
class RenderRequest;

inline mbgl::util::RunLoop& threadRunLoop() {
    // MapLibre Native's RunLoop is thread-affine. Keep one private loop per
    // renderer-owning thread and share it between renderers on that thread.
    thread_local mbgl::util::RunLoop loop(mbgl::util::RunLoop::Type::New);
    return loop;
}

inline void bindThreadRunLoop() {
    mbgl::Scheduler::SetCurrent(&threadRunLoop());
}

inline void currentThreadRunLoopTick() {
    // Tick can be driven through a Rust handle without constructing a renderer first.
    bindThreadRunLoop();
    threadRunLoop().runOnce();
}

inline std::unique_ptr<std::string> encodeImage(mbgl::PremultipliedImage image) {
    auto unpremultipliedImage = mbgl::util::unpremultiply(std::move(image));

    const size_t pixelCount = unpremultipliedImage.size.width * unpremultipliedImage.size.height;
    std::string data;
    data.reserve(2 * sizeof(uint32_t) + pixelCount * BYTES_PER_PIXEL);

    uint32_t width = unpremultipliedImage.size.width;
    uint32_t height = unpremultipliedImage.size.height;
    data.append(reinterpret_cast<const char*>(&width), sizeof(uint32_t));
    data.append(reinterpret_cast<const char*>(&height), sizeof(uint32_t));

    const char* pixelData = reinterpret_cast<const char*>(unpremultipliedImage.data.get());
    data.append(pixelData, pixelCount * BYTES_PER_PIXEL);

    return std::make_unique<std::string>(std::move(data));
}

class MapRenderer {
public:
    explicit MapRenderer(mbgl::MapMode mapMode,
                         mbgl::Size size,
                         float pixelRatio,
                         const mbgl::ResourceOptions& resourceOptions)
        : mapObserverInstance(std::make_shared<MapObserver>()) {
        bindThreadRunLoop();
        frontend = std::make_unique<mbgl::HeadlessFrontend>(size, pixelRatio);

        mbgl::MapOptions mapOptions;
        mapOptions.withMapMode(mapMode).withSize(size).withPixelRatio(pixelRatio);

        // Set up logging observer for Rust bridge
        auto logObserver = std::make_unique<mln::bridge::RustLogObserver>();
        mbgl::Log::setObserver(std::move(logObserver));
        map = std::make_unique<mbgl::Map>(*frontend, *mapObserverInstance, mapOptions, resourceOptions);
    }
    ~MapRenderer() {}

    std::shared_ptr<MapObserver> observer() {
        return mapObserverInstance;
    }

    #if defined(MLN_WEBGPU_IMPL_FFI)
    std::shared_ptr<mbgl::webgpu::Texture2D> takeTexture() {
        auto backend = static_cast<mbgl::webgpu::HeadlessBackend*>(this->frontend->getBackend());
        auto ptr = std::static_pointer_cast<mbgl::webgpu::Texture2D>(backend->takeTexture());
        assert(ptr);
        return ptr;
    }
    #endif

    void style_add_image(rust::Str id,
                         rust::Slice<const unsigned char> data,
                         mbgl::Size size,
                         bool signed_distance_field) {
        mbgl::PremultipliedImage image(size, data.data(), data.size());

        const float pixelRatio = 1.0;
        map->getStyle().addImage(std::make_unique<mbgl::style::Image>(
            std::string(id), std::move(image), pixelRatio, signed_distance_field));
    }

    void style_remove_image(rust::Str id) {
        map->getStyle().removeImage(std::string(id));
    }

    void style_add_source(std::unique_ptr<mbgl::style::Source> source) {
        map->getStyle().addSource(std::move(source));
    }

    void style_remove_source(rust::Str id) {
        map->getStyle().removeSource(std::string(id));
    }

    void style_add_layer(std::unique_ptr<mbgl::style::Layer> layer, rust::Str before_id) {
        // An empty before_id string means no before layer was specified.
        map->getStyle().addLayer(
            std::move(layer),
            before_id.empty() ? std::nullopt : std::optional<std::string>{std::string(before_id)});
    }

    void style_remove_layer(rust::Str id) {
        map->getStyle().removeLayer(std::string(id));
    }

    void style_load_from_url(const rust::Str styleUrl) {
        map->getStyle().loadURL((std::string)styleUrl);
    }

    void style_load_from_json(const rust::Str styleJson) {
        map->getStyle().loadJSON((std::string)styleJson);
    }

    std::unique_ptr<BridgeImage> readStillImage() {
        auto image = frontend->readStillImage();
        auto unpremultipliedImage = mbgl::util::unpremultiply(std::move(image));
        return std::make_unique<BridgeImage>(std::move(unpremultipliedImage.data), unpremultipliedImage.size);
    }

    void render_once() {
        frontend->renderOnce(*map);
    }

    std::unique_ptr<RenderRequest> submitRender(double lat, double lon, double zoom, double bearing, double pitch);

    std::unique_ptr<std::string> readStillImageBytes() {
        return encodeImage(frontend->readStillImage());
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

    void pitchBy(double pitch) {
        map->pitchBy(pitch);
    }

    void rotateBy(const mbgl::ScreenCoordinate& first, const mbgl::ScreenCoordinate& second) {
        map->rotateBy(first, second);
    }

    // Set the wgpu device and queue required for rendering when using the wgpu ffi backend
    #if defined(MLN_WEBGPU_IMPL_FFI)
    void setDeviceAndQueue(WGPUDevice device, WGPUQueue queue) {
        static_cast<mbgl::webgpu::RendererBackend*>(frontend->getBackend())->setDevice(device);
        static_cast<mbgl::webgpu::RendererBackend*>(frontend->getBackend())->setQueue(queue);
    }
    #endif
public:
    // CXX bridge helpers below access these directly. Keep them alive here
    // because the frontend and observer are passed by reference to the map.
    std::unique_ptr<mbgl::HeadlessFrontend> frontend;
    std::shared_ptr<MapObserver> mapObserverInstance;
    std::unique_ptr<mbgl::Map> map;
};

class RenderRequest {
public:
    struct State {
        bool ready = false;
        std::exception_ptr error;
        std::unique_ptr<std::string> image;
    };

    RenderRequest()
        : state(std::make_shared<State>()) {}

    ~RenderRequest() {
        // If the request is dropped before completion, drive the run loop here
        // while the borrowed renderer is still alive, so MapLibre Native returns to
        // an idle state before the next submitRender.
        while (!state->ready) {
            currentThreadRunLoopTick();
        }
    }

    std::shared_ptr<State> getState() const {
        return state;
    }

    bool isReady() const {
        return state->ready;
    }

    bool hasError() const {
        return static_cast<bool>(state->error);
    }

    rust::String errorMessage() const {
        if (!state->error) {
            return rust::String();
        }

        try {
            std::rethrow_exception(state->error);
        } catch (const std::exception& error) {
            return rust::String(error.what());
        } catch (...) {
            return rust::String("Unknown render error");
        }
    }

    std::unique_ptr<std::string> takeImage() {
        assert(state->ready);
        assert(!state->error);
        assert(state->image);
        assert(!taken);
        taken = true;
        return std::move(state->image);
    }

private:
    std::shared_ptr<State> state;
    bool taken = false;
};

inline std::unique_ptr<RenderRequest> MapRenderer::submitRender(
        double lat,
        double lon,
        double zoom,
        double bearing,
        double pitch) {
    setCamera(lat, lon, zoom, bearing, pitch);

    auto request = std::make_unique<RenderRequest>();
    auto state = request->getState();

    map->renderStill([this, state](const std::exception_ptr& error) {
        state->error = error;
        if (!error) {
            state->image = readStillImageBytes();
        }
        state->ready = true;
    });

    return request;
}

inline std::unique_ptr<MapRenderer> MapRenderer_new(
            mbgl::MapMode mapMode,
            uint32_t width,
            uint32_t height,
            float pixelRatio,
            const mbgl::ResourceOptions& resourceOptions
) {
    mbgl::Size size = {width, height};
    return std::make_unique<MapRenderer>(mapMode, size, pixelRatio, resourceOptions);
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
