#include "texture.h"
#include <memory>

namespace mln::bridge::texture {
    WGPUTexture getWGPUTexture(const std::shared_ptr<mbgl::webgpu::Texture2D>& texture2d) {
        if (!texture2d) {
            return nullptr;
        }
        WGPUTexture texture = texture2d->getTexture();
        return texture;
    }
}
