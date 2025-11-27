#pragma once

#include <memory>
#include <functional>
#include <mbgl/renderer/renderer_observer.hpp>
#include "maplibre_native/src/renderer/bridge.rs.h"

namespace mln {
namespace bridge {

using RendererObserverCallback = void(*)();

// TODO: why those 2 are required?
extern void render_observer_notification() noexcept;

class CustomRendererObserver: public mbgl::RendererObserver {
public:
    explicit CustomRendererObserver(RendererObserverCallback callback)
        : m_callback(callback) {}

    void onInvalidate() override {
        if (m_callback)
            render_observer_notification();
    }

    void onDidFinishRenderingFrame(mbgl::RendererObserver::RenderMode /*mode*/, bool needsRepaint,
                                   bool placementChanged) override {
        if (needsRepaint || placementChanged) {
            onInvalidate();
        }
    }

    

private:
   RendererObserverCallback m_callback; 
};

inline std::unique_ptr<mbgl::RendererObserver> RendererObserver_create_observer(RendererObserverCallback callback) {
    return std::unique_ptr<mbgl::RendererObserver>(new CustomRendererObserver(callback));
}

} // namespace bridge
} // namespace mln