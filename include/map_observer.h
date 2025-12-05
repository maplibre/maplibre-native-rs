#pragma once

#include "rust/cxx.h"
#include <mbgl/map/map_observer.hpp>
#include <memory>
#include "bridge.h"

namespace mln {
namespace bridge {

    using MapObserverCameraChangeMode = mbgl::MapObserver::CameraChangeMode; // Required, because enum nested in class is not supported by cxx

    struct FailingLoadingMapTrampoline {
        void call(mbgl::MapLoadError error, const std::string& what) {
            if (trampoline_function) {
                trampoline_function(function_pointer, error, what);
            }
        }
        private:
            void(*trampoline_function)(std::int8_t* function_pointer, mbgl::MapLoadError, const std::string&){nullptr};
            std::int8_t* function_pointer{nullptr};
    };

    struct CameraDidChangeTrampoline {
        void call(MapObserverCameraChangeMode mode) {
            if (trampoline_function) {
                trampoline_function(function_pointer, mode);
            }
        }
        private:
            void(*trampoline_function)(std::int8_t* function_pointer, MapObserverCameraChangeMode mode) {nullptr};
            std::int8_t* function_pointer{nullptr};
    };

    struct DidFinishRenderingFrameTrampoline {
        void call(bool needsRepaint, bool placementChanged) {
            if (trampoline_function) {
                trampoline_function(function_pointer, needsRepaint, placementChanged);
            }
        }
        private:
            void(*trampoline_function)(std::int8_t* function_pointer, bool needsRepaint, bool placementChanged) {nullptr};
            std::int8_t* function_pointer{nullptr};
    };

    class MapObserver: public mbgl::MapObserver {
        public:
            void setOnWillStartLoadingMapCallback(VoidTrampoline callback) {
                onWillStartLoadingMapCallback = callback;
            }

            void setOnDidFinishLoadingStyleCallback(VoidTrampoline callback) {
                onWillStartLoadingMapCallback = callback;
            }

            void setOnDidBecomeIdleCallback(VoidTrampoline callback) {
                onWillStartLoadingMapCallback = callback;
            }

            void setOnDidFailLoadingMapCallback(FailingLoadingMapTrampoline callback) {
                onDidFailLoadingMapCallback = callback;
            }

            void setDidFinishRenderingFrameCallback(DidFinishRenderingFrameTrampoline callback) {
                onDidFinishRenderingFrameCallback = callback;
            }

        private:
            void onWillStartLoadingMap() override {
                onWillStartLoadingMapCallback.call();
            }
            void onDidFinishLoadingStyle() override {
                onDidFinishLoadingStyleCallback.call();
            }
            void onDidBecomeIdle() override {
                onDidBecomeIdleCallback.call();
            }

            void onDidFailLoadingMap(mbgl::MapLoadError error, const std::string& what) override {
                onDidFailLoadingMapCallback.call(error, what);
            }

            void onCameraDidChange(MapObserverCameraChangeMode mode) override {
                onCameraDidChangeCallback.call(mode);
            }
            // void onSourceChanged(mbgl::style::Source&) override;

            void onDidFinishRenderingFrame(const mbgl::MapObserver::RenderFrameStatus& status) override {
                onDidFinishRenderingFrameCallback.call(status.needsRepaint, status.placementChanged);
            }

        private:
            VoidTrampoline onWillStartLoadingMapCallback;
            VoidTrampoline onDidFinishLoadingStyleCallback;
            VoidTrampoline onDidBecomeIdleCallback;
            FailingLoadingMapTrampoline onDidFailLoadingMapCallback;
            CameraDidChangeTrampoline onCameraDidChangeCallback;
            DidFinishRenderingFrameTrampoline onDidFinishRenderingFrameCallback;
    };

    inline std::unique_ptr<MapObserver> MapObserver_create_observer() {
        return std::unique_ptr<MapObserver>(new MapObserver());
    }

} // namespace bridge
} // namespace mln
