#pragma once

#include <memory>
#include <functional>
#include <mbgl/renderer/renderer_observer.hpp>
#include "maplibre_native/src/renderer/bridge.rs.h"

namespace mln {
namespace bridge {

// Forward declarations
class FinishRenderingFrameCallback; // Required, because this file gets included into the bridge.rs file and therefore added to the mablibre_native/bridige.rs.h
void finish_rendering_frame_callback(FinishRenderingFrameCallback const &callback, bool, bool) noexcept;

class CustomRendererObserver: public mbgl::RendererObserver {
public:
    explicit CustomRendererObserver(rust::Box<FinishRenderingFrameCallback> callback)
        : m_callback(std::move(callback)) {}
    CustomRendererObserver() = delete;

    void onDidFinishRenderingFrame(mbgl::RendererObserver::RenderMode /*mode*/, bool needsRepaint, bool placementChanged) override {
        finish_rendering_frame_callback(*m_callback, needsRepaint, placementChanged);
    }
private:
   rust::Box<FinishRenderingFrameCallback> m_callback;
};

inline std::unique_ptr<mbgl::RendererObserver> RendererObserver_create_observer(rust::Box<FinishRenderingFrameCallback> callback) {
    return std::unique_ptr<mbgl::RendererObserver>(new CustomRendererObserver(std::move(callback)));
}

} // namespace bridge
} // namespace mln
