#pragma once

#include <memory>
#include <functional>
#include <mbgl/renderer/renderer_observer.hpp>
#include "maplibre_native/src/renderer/bridge.rs.h"
#include "bridge.h"

namespace mln {
namespace bridge {

class CustomRendererObserver: public mbgl::RendererObserver {
public:
    explicit CustomRendererObserver(VoidTrampoline callback)
        : m_callback(callback) {}
    CustomRendererObserver() = delete;

    void onInvalidate() override {
        m_callback.call();
    }

    void onDidFinishRenderingFrame(mbgl::RendererObserver::RenderMode /*mode*/, bool needsRepaint,
                                   bool placementChanged) override {
        if (needsRepaint || placementChanged) {
            onInvalidate();
        }
    }
private:
   VoidTrampoline m_callback;
};

inline std::unique_ptr<mbgl::RendererObserver> RendererObserver_create_observer(VoidTrampoline trampoline) {
    return std::unique_ptr<mbgl::RendererObserver>(new CustomRendererObserver(trampoline));
}

} // namespace bridge
} // namespace mln
