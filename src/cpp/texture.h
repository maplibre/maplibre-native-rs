#pragma once

#if defined(MLN_WEBGPU_IMPL_FFI)

#include <mbgl/webgpu/texture2d.hpp>
#include <cstdint>

namespace mln::bridge::texture {
    WGPUTexture getWGPUTexture(const std::shared_ptr<mbgl::webgpu::Texture2D>&);
}

#endif // #if defined(MLN_WEBGPU_IMPL_FFI)
