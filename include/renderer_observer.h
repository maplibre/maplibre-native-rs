#pragma once

#include <memory>
#include <functional>
#include <mbgl/renderer/renderer_observer.hpp>
#include "maplibre_native/src/renderer/bridge.rs.h"

namespace mln {
namespace bridge {

class VoidCallback; // Required, because this file gets included into the bridge.rs file and therefore added to the mablibre_native/bridige.rs.h
void void_callback(VoidCallback const &trampoline) noexcept;

class CustomRendererObserver: public mbgl::RendererObserver {
public:
    explicit CustomRendererObserver(rust::Box<VoidCallback> callback)
        : m_callback(std::move(callback)) {}
    CustomRendererObserver() = delete;

    void onInvalidate() override {
        void_callback(*m_callback);
    }

    void onDidFinishRenderingFrame(mbgl::RendererObserver::RenderMode /*mode*/, bool needsRepaint,
                                   bool placementChanged) override {
        if (needsRepaint || placementChanged) {
            onInvalidate();
        }
    }
private:
   rust::Box<VoidCallback> m_callback;
};

inline std::unique_ptr<mbgl::RendererObserver> RendererObserver_create_observer(rust::Box<VoidCallback> payload) {
    return std::unique_ptr<mbgl::RendererObserver>(new CustomRendererObserver(std::move(payload)));
}

} // namespace bridge
} // namespace mln