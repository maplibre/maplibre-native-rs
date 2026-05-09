#include "texture.h"
#include <memory>

namespace mln::bridge::texture {
    WGPUTexture getRawTextureHandle(const std::shared_ptr<mbgl::webgpu::Texture2D>& texture2d) {
        if (!texture2d) {
            return nullptr;
        }
        WGPUTexture texture = texture2d->getTexture();
        if (!texture) {
            return nullptr;
        }
        // Retain so Rust can safely clone and then release its temporary handle.
        wgpuTextureAddRef(texture);
        return texture;
    }
}
