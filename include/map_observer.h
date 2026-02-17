#pragma once

#include "rust/cxx.h"
#include <mbgl/map/map_observer.hpp>
#include <memory>
#include <optional>

namespace mln {
namespace bridge {

    // Forward declarations
    using MapObserverCameraChangeMode = mbgl::MapObserver::CameraChangeMode; // Required, because enum nested in class is not supported by cxx

    class VoidCallback;
    void void_callback(VoidCallback const& callback) noexcept;

    class FailingLoadingMapCallback;
    void failing_loading_map_callback(FailingLoadingMapCallback const& callback, mbgl::MapLoadError error, const rust::Str what) noexcept;

    class CameraDidChangeCallback;
    void camera_did_change_callback(CameraDidChangeCallback const& callback, MapObserverCameraChangeMode mode) noexcept;

    class FinishRenderingFrameCallback;
    void finish_rendering_frame_callback(FinishRenderingFrameCallback const& callback, bool needsRepaint, bool placementChanged) noexcept;

    class MapObserver: public mbgl::MapObserver {
        public:
            void setWillStartLoadingMapCallback(rust::Box<VoidCallback> callback) {
                willStartLoadingMapCallback = std::optional<rust::Box<VoidCallback>>{std::move(callback)};
            }

            void setFinishLoadingStyleCallback(rust::Box<VoidCallback> callback) {
                finishLoadingStyleCallback = std::optional<rust::Box<VoidCallback>>{std::move(callback)};
            }

            void setBecomeIdleCallback(rust::Box<VoidCallback> callback) {
                becomeIdleCallback = std::optional<rust::Box<VoidCallback>>{std::move(callback)};
            }

            void setFailLoadingMapCallback(rust::Box<FailingLoadingMapCallback> callback) {
                failLoadingMapCallback = std::optional<rust::Box<FailingLoadingMapCallback>>{std::move(callback)};
            }

            void setFinishRenderingFrameCallback(rust::Box<FinishRenderingFrameCallback> callback) {
                finishRenderingFrameCallback = std::optional<rust::Box<FinishRenderingFrameCallback>>{std::move(callback)};
            }

            void setCameraDidChangeCallback(rust::Box<CameraDidChangeCallback> callback) {
                cameraDidChangeCallback = std::optional<rust::Box<CameraDidChangeCallback>>{std::move(callback)};
            }

        private:
            void onWillStartLoadingMap() override {
                try {
                    void_callback(*willStartLoadingMapCallback.value());
                } catch (...) {}
            }
            void onDidFinishLoadingStyle() override {
                try {
                    void_callback(*(finishLoadingStyleCallback.value()));
                } catch (...) {}
            }
            void onDidBecomeIdle() override {
                try {
                    void_callback(*(becomeIdleCallback.value()));
                } catch (...) {}
            }

            void onDidFailLoadingMap(mbgl::MapLoadError error, const std::string& what) override {
                try {
                    failing_loading_map_callback(*(failLoadingMapCallback.value()), error, what);
                } catch (...) {}
            }

            void onCameraDidChange(MapObserverCameraChangeMode mode) override {
                try {
                    camera_did_change_callback(*(cameraDidChangeCallback.value()), mode);
                } catch (...) {}
            }
            // void onSourceChanged(mbgl::style::Source&) override;

            void onDidFinishRenderingFrame(const mbgl::MapObserver::RenderFrameStatus& status) override {
                try {
                    finish_rendering_frame_callback(*(finishRenderingFrameCallback.value()), status.needsRepaint, status.placementChanged);
                } catch (...) {}
            }

        private:
            std::optional<rust::Box<VoidCallback>> willStartLoadingMapCallback;
            std::optional<rust::Box<VoidCallback>> finishLoadingStyleCallback;
            std::optional<rust::Box<VoidCallback>> becomeIdleCallback;
            std::optional<rust::Box<FailingLoadingMapCallback>> failLoadingMapCallback;
            std::optional<rust::Box<CameraDidChangeCallback>> cameraDidChangeCallback;
            std::optional<rust::Box<FinishRenderingFrameCallback>> finishRenderingFrameCallback;
    };

} // namespace bridge
} // namespace mln
