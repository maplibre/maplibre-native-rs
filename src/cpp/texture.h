#pragma once

#include <mbgl/webgpu/texture2d.hpp>
#include <cstdint>

namespace mln::bridge::texture {
    WGPUExtent3D getExtend3d(const std::shared_ptr<mbgl::webgpu::Texture2D>&);
    uint32_t getMipLevelCount(const std::shared_ptr<mbgl::webgpu::Texture2D>&);
    uint32_t getSampleCount(const std::shared_ptr<mbgl::webgpu::Texture2D>&);
    WGPUTextureDimension getDimension(const std::shared_ptr<mbgl::webgpu::Texture2D>&);
    WGPUTextureFormat getFormat(const std::shared_ptr<mbgl::webgpu::Texture2D>&);
    WGPUTextureUsage getUsage(const std::shared_ptr<mbgl::webgpu::Texture2D>&);
    uintptr_t getRawTextureHandle(const std::shared_ptr<mbgl::webgpu::Texture2D>&);
}
