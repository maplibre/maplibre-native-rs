#pragma once

#include <mbgl/webgpu/texture2d.hpp>
#include <cstdint>

namespace mln::bridge::texture {
    WGPUTexture getRawTextureHandle(const std::shared_ptr<mbgl::webgpu::Texture2D>&);
}
