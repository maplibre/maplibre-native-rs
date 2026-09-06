#pragma once

#if defined(MLN_WEBGPU_IMPL_FFI)

#include <mln/webgpu/texture2d.hpp>
#include <cstdint>

namespace mln::bridge::texture {
    WGPUTexture getWGPUTexture(const std::shared_ptr<mln::webgpu::Texture2D>&);
}

#endif // #if defined(MLN_WEBGPU_IMPL_FFI)
