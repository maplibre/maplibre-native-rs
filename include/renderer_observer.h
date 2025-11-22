#pragma once

#include <memory>
#include <functional>
#include <mbgl/renderer/renderer_observer.hpp>

namespace mln::bridge {

class RendererObserverMy: public mbgl::RendererObserver {
public:
    explicit RendererObserverMy(std::function<void()> notifyRepaint)
        : m_notifyRepaint(std::move(notifyRepaint)) {}

private:
   std::function<void()> m_notifyRepaint; 
};

inline std::unique_ptr<mbgl::RendererObserver> RendererObserver_create_observer() {
    return nullptr;
}

} // namespace mln::bridge