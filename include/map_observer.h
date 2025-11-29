#pragma once

#include "rust/cxx.h"
#include <mbgl/map/map_observer.hpp>
#include <memory>

namespace mln {
namespace bridge {

    using MapObserverCameraChangeMode = mbgl::MapObserver::CameraChangeMode; // Required, because enum nested in class is not supported by cxx
    using EmptyCallback = void(*)();
    using FailingLoadingMapCallback = void(*)(mbgl::MapLoadError, const std::string&);
    using CameraDidChangeCallback = void(*)(MapObserverCameraChangeMode mode);
    using DidFinishRenderingFrameCallback = void(*)(bool needsRepaint, bool placementChanged);

    class MapObserver: public mbgl::MapObserver {
        public:
            void setOnWillStartLoadingMapCallback(EmptyCallback callback) {
                onWillStartLoadingMapCallback = callback;
            }

            void setOnDidFinishLoadingStyleCallback(EmptyCallback callback) {
                onWillStartLoadingMapCallback = callback;
            }

            void setOnDidBecomeIdleCallback(EmptyCallback callback) {
                onWillStartLoadingMapCallback = callback;
            }

            void setOnDidFailLoadingMapCallback(FailingLoadingMapCallback callback) {
                onDidFailLoadingMapCallback = callback;
            }

            void setDidFinishRenderingFrameCallback(DidFinishRenderingFrameCallback callback) {
                onDidFinishRenderingFrameCallback = callback;
            }

        private:
            void onWillStartLoadingMap() override {
                if (onWillStartLoadingMapCallback) {
                    onWillStartLoadingMapCallback();
                }
            }
            void onDidFinishLoadingStyle() override {
                if (onDidFinishLoadingStyleCallback) {
                    onDidFinishLoadingStyleCallback();
                }
            }
            void onDidBecomeIdle() override {
                if (onDidBecomeIdleCallback) {
                    onDidBecomeIdleCallback();
                }
            }

            void onDidFailLoadingMap(mbgl::MapLoadError error, const std::string& what) override {
                if (onDidFailLoadingMapCallback) {
                    onDidFailLoadingMapCallback(error, what);
                }
            }

            void onCameraDidChange(MapObserverCameraChangeMode mode) override {
                if (onCameraDidChangeCallback) {
                    onCameraDidChangeCallback(mode);
                }
            }
            // void onSourceChanged(mbgl::style::Source&) override;

            void onDidFinishRenderingFrame(const mbgl::MapObserver::RenderFrameStatus& status) override {
                if (onDidFinishRenderingFrameCallback) {
                    onDidFinishRenderingFrameCallback(status.needsRepaint, status.placementChanged);
                }
            }

        private:
            EmptyCallback onWillStartLoadingMapCallback{nullptr};
            EmptyCallback onDidFinishLoadingStyleCallback{nullptr};
            EmptyCallback onDidBecomeIdleCallback{nullptr};
            FailingLoadingMapCallback onDidFailLoadingMapCallback{nullptr};
            CameraDidChangeCallback onCameraDidChangeCallback{nullptr};
            DidFinishRenderingFrameCallback onDidFinishRenderingFrameCallback{nullptr};
    };

    inline std::unique_ptr<MapObserver> MapObserver_create_observer() {
        return std::unique_ptr<MapObserver>(new MapObserver());
    }

} // namespace bridge
} // namespace mln