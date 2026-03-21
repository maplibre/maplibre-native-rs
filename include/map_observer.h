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
            void setWillStartLoadingMapCallback(rust::Box<VoidCallback> callback) const {
                willStartLoadingMapCallback = std::optional<rust::Box<VoidCallback>>{std::move(callback)};
            }

            void setFinishLoadingStyleCallback(rust::Box<VoidCallback> callback) const {
                finishLoadingStyleCallback = std::optional<rust::Box<VoidCallback>>{std::move(callback)};
            }

            void setBecomeIdleCallback(rust::Box<VoidCallback> callback) const {
                becomeIdleCallback = std::optional<rust::Box<VoidCallback>>{std::move(callback)};
            }

            void setFailLoadingMapCallback(rust::Box<FailingLoadingMapCallback> callback) const {
                failLoadingMapCallback = std::optional<rust::Box<FailingLoadingMapCallback>>{std::move(callback)};
            }

            void setFinishRenderingFrameCallback(rust::Box<FinishRenderingFrameCallback> callback) const {
                finishRenderingFrameCallback = std::optional<rust::Box<FinishRenderingFrameCallback>>{std::move(callback)};
            }

            void setCameraDidChangeCallback(rust::Box<CameraDidChangeCallback> callback) const {
                cameraDidChangeCallback = std::optional<rust::Box<CameraDidChangeCallback>>{std::move(callback)};
            }

        private:
            void onWillStartLoadingMap() override {
                if (willStartLoadingMapCallback.has_value()) {
                    void_callback(*(*willStartLoadingMapCallback)); 
                }
            }
            void onDidFinishLoadingStyle() override {
                if (finishLoadingStyleCallback.has_value()) {
                    void_callback(*(*finishLoadingStyleCallback)); 
                }
            }
            void onDidBecomeIdle() override {
                if (becomeIdleCallback.has_value()) {
                    void_callback(*(*becomeIdleCallback)); 
                }
            }

            void onDidFailLoadingMap(mbgl::MapLoadError error, const std::string& what) override {
                if (failLoadingMapCallback.has_value()) {
                    failing_loading_map_callback(*(*failLoadingMapCallback), error, what);
                }
            }

            void onCameraDidChange(MapObserverCameraChangeMode mode) override {
                if (cameraDidChangeCallback.has_value()) {
                    camera_did_change_callback(*(*cameraDidChangeCallback), mode);
                }
            }
            // void onSourceChanged(mbgl::style::Source&) override;

            void onDidFinishRenderingFrame(const mbgl::MapObserver::RenderFrameStatus& status) override {
                if (finishRenderingFrameCallback.has_value()) {
                    finish_rendering_frame_callback(*(*finishRenderingFrameCallback), status.needsRepaint, status.placementChanged);
                }
            }

        private:
            mutable std::optional<rust::Box<VoidCallback>> willStartLoadingMapCallback;
            mutable std::optional<rust::Box<VoidCallback>> finishLoadingStyleCallback;
            mutable std::optional<rust::Box<VoidCallback>> becomeIdleCallback;
            mutable std::optional<rust::Box<FailingLoadingMapCallback>> failLoadingMapCallback;
            mutable std::optional<rust::Box<CameraDidChangeCallback>> cameraDidChangeCallback;
            mutable std::optional<rust::Box<FinishRenderingFrameCallback>> finishRenderingFrameCallback;
    };

} // namespace bridge
} // namespace mln
