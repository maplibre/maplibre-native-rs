#include "texture.h"
#include <memory>

namespace mln::bridge::texture {
    WGPUExtent3D getExtend3d(const std::shared_ptr<mbgl::webgpu::Texture2D>& texture2d) {

        return WGPUExtent3D {
            wgpuTextureGetWidth(texture2d->getTexture()),
            wgpuTextureGetHeight(texture2d->getTexture()),
            wgpuTextureGetDepthOrArrayLayers(texture2d->getTexture())
        };
    }

    uint32_t getMipLevelCount(const std::shared_ptr<mbgl::webgpu::Texture2D>& texture2d) {
        return wgpuTextureGetMipLevelCount(texture2d->getTexture());
    }
    uint32_t getSampleCount(const std::shared_ptr<mbgl::webgpu::Texture2D>& texture2d) {
        return wgpuTextureGetSampleCount(texture2d->getTexture());
    }

    WGPUTextureDimension getDimension(const std::shared_ptr<mbgl::webgpu::Texture2D>& texture2d) {
        return wgpuTextureGetDimension(texture2d->getTexture());
    }

    WGPUTextureFormat getFormat(const std::shared_ptr<mbgl::webgpu::Texture2D>& texture2d) {
        return wgpuTextureGetFormat(texture2d->getTexture());
    }

    WGPUTextureUsage getUsage(const std::shared_ptr<mbgl::webgpu::Texture2D>& texture2d) {
        return wgpuTextureGetUsage(texture2d->getTexture());
    }
}


// struct Texture {
// public:
//     Texture() = delete;
//     Texture(WGPUTexture texture): mTexture(texture) {
//         assert(mTexture);
//     }

//     std::unique_ptr<TextureView> createView(WGPUTextureFormat format, WGPUTextureViewDimension dimension, WGPUTextureUsage usage, WGPUTextureAspect aspect, uint32_t base_mip_level, uint32_t mip_level_count, uint32_t base_array_layer, uint32_t array_layer_count) const {
//         const auto desc = WGPUTextureViewDescriptor {
//             .nextInChain = nullptr,
//             .label = WGPUStringView {nullptr, 0},
//             .format = format,
//             .dimension = dimension,
//             .baseMipLevel = base_mip_level,
//             .mipLevelCount = mip_level_count,
//             .baseArrayLayer = base_array_layer,
//             .arrayLayerCount = array_layer_count,
//             .aspect = aspect,
//             .usage = usage,
//         };
//         return std::make_unique<TextureView>(wgpuTextureCreateView(mTexture, &desc));
//     }

//     ~Texture() {
//         destroy();
//     }

//     void destroy() const {
//         wgpuTextureDestroy(mTexture);
//         // mTexture = nullptr;
//     }
// private:
//     WGPUTexture mTexture;
// };

// struct TextureView {
// public:
//     TextureView() = delete;
//     explicit TextureView(WGPUTextureView textureView): mTextureView(textureView) {
//         assert(mTextureView);
//     }

//     ~TextureView() {
//         if (mTextureView) {
//             wgpuTextureViewRelease(mTextureView);
//         }
//     }

// private:
//     WGPUTextureView mTextureView;
// };
