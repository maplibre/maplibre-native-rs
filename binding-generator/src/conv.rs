use wgpu::{
    AddressMode, BindGroupLayoutEntry, BindingType, BufferBindingType, BufferDescriptor,
    BufferUsages, CommandEncoderDescriptor, CompareFunction, Extent3d, FilterMode,
    MipmapFilterMode, SamplerBindingType, SamplerDescriptor, ShaderStages, StorageTextureAccess,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureViewDescriptor, TextureViewDimension,
};

use std::num::{NonZeroU32, NonZeroU64};

use crate::{
    WGPU_ARRAY_LAYER_COUNT_UNDEFINED, WGPU_MIP_LEVEL_COUNT_UNDEFINED, WGPUAddressMode_MirrorRepeat,
    WGPUAddressMode_Repeat, WGPUBufferDescriptor, WGPUCommandEncoderDescriptor,
    WGPUCompareFunction_Always, WGPUCompareFunction_Equal, WGPUCompareFunction_Greater,
    WGPUCompareFunction_GreaterEqual, WGPUCompareFunction_Less, WGPUCompareFunction_LessEqual,
    WGPUCompareFunction_Never, WGPUCompareFunction_NotEqual, WGPUFilterMode_Linear,
    WGPUMipmapFilterMode_Linear, WGPUSamplerDescriptor, WGPUStringView,
    WGPUTextureAspect_DepthOnly, WGPUTextureAspect_StencilOnly, WGPUTextureAspect_Undefined,
    WGPUTextureDescriptor, WGPUTextureDimension_1D, WGPUTextureDimension_3D,
    WGPUTextureFormat_BGRA8Unorm, WGPUTextureFormat_BGRA8UnormSrgb, WGPUTextureFormat_Depth16Unorm,
    WGPUTextureFormat_Depth24Plus, WGPUTextureFormat_Depth24PlusStencil8,
    WGPUTextureFormat_Depth32Float, WGPUTextureFormat_Depth32FloatStencil8,
    WGPUTextureFormat_R8Sint, WGPUTextureFormat_R8Snorm, WGPUTextureFormat_R8Uint,
    WGPUTextureFormat_R8Unorm, WGPUTextureFormat_R16Float, WGPUTextureFormat_R16Sint,
    WGPUTextureFormat_R16Snorm, WGPUTextureFormat_R16Uint, WGPUTextureFormat_R16Unorm,
    WGPUTextureFormat_R32Float, WGPUTextureFormat_R32Sint, WGPUTextureFormat_R32Uint,
    WGPUTextureFormat_RG8Sint, WGPUTextureFormat_RG8Snorm, WGPUTextureFormat_RG8Uint,
    WGPUTextureFormat_RG8Unorm, WGPUTextureFormat_RG11B10Ufloat, WGPUTextureFormat_RG16Float,
    WGPUTextureFormat_RG16Sint, WGPUTextureFormat_RG16Snorm, WGPUTextureFormat_RG16Uint,
    WGPUTextureFormat_RG16Unorm, WGPUTextureFormat_RG32Float, WGPUTextureFormat_RG32Sint,
    WGPUTextureFormat_RG32Uint, WGPUTextureFormat_RGB9E5Ufloat, WGPUTextureFormat_RGB10A2Uint,
    WGPUTextureFormat_RGB10A2Unorm, WGPUTextureFormat_RGBA8Sint, WGPUTextureFormat_RGBA8Snorm,
    WGPUTextureFormat_RGBA8Uint, WGPUTextureFormat_RGBA8Unorm, WGPUTextureFormat_RGBA8UnormSrgb,
    WGPUTextureFormat_RGBA16Float, WGPUTextureFormat_RGBA16Sint, WGPUTextureFormat_RGBA16Snorm,
    WGPUTextureFormat_RGBA16Uint, WGPUTextureFormat_RGBA16Unorm, WGPUTextureFormat_RGBA32Float,
    WGPUTextureFormat_RGBA32Sint, WGPUTextureFormat_RGBA32Uint, WGPUTextureFormat_Stencil8,
    WGPUTextureFormat_Undefined, WGPUTextureUsage_None, WGPUTextureViewDescriptor,
    WGPUTextureViewDimension_1D, WGPUTextureViewDimension_2D, WGPUTextureViewDimension_2DArray,
    WGPUTextureViewDimension_3D, WGPUTextureViewDimension_Cube, WGPUTextureViewDimension_CubeArray,
    WGPUTextureViewDimension_Undefined,
};

/// Convert a `WGPUStringView` to an `Option<&str>`.
/// Returns `None` for the null sentinel `{NULL, WGPU_STRLEN}`.
///
/// # Safety
/// The caller must ensure that `view.data` points to valid UTF-8 memory for
/// `view.length` bytes (or is null-terminated when `length == WGPU_STRLEN`).
pub unsafe fn string_view<'a>(view: WGPUStringView) -> Option<&'a str> {
    if view.data.is_null() {
        return None;
    }
    let bytes = if view.length == usize::MAX {
        // WGPU_STRLEN — treat as null-terminated
        unsafe { std::ffi::CStr::from_ptr(view.data) }.to_bytes()
    } else {
        unsafe { std::slice::from_raw_parts(view.data as *const u8, view.length) }
    };
    std::str::from_utf8(bytes).ok()
}

pub fn sampler_descriptor<'a>(d: &'a WGPUSamplerDescriptor) -> SamplerDescriptor<'a> {
    SamplerDescriptor {
        label: unsafe { string_view(d.label) },
        address_mode_u: map_address_mode(d.addressModeU),
        address_mode_v: map_address_mode(d.addressModeV),
        address_mode_w: map_address_mode(d.addressModeW),
        mag_filter: map_filter_mode(d.magFilter),
        min_filter: map_filter_mode(d.minFilter),
        mipmap_filter: map_mipmap_filter_mode(d.mipmapFilter),
        lod_min_clamp: d.lodMinClamp,
        lod_max_clamp: d.lodMaxClamp,
        compare: map_compare_function(d.compare),
        anisotropy_clamp: d.maxAnisotropy,
        border_color: None,
    }
}

fn map_address_mode(mode: crate::WGPUAddressMode) -> AddressMode {
    match mode {
        WGPUAddressMode_Repeat => AddressMode::Repeat,
        WGPUAddressMode_MirrorRepeat => AddressMode::MirrorRepeat,
        _ => AddressMode::ClampToEdge,
    }
}

fn map_filter_mode(mode: crate::WGPUFilterMode) -> FilterMode {
    match mode {
        WGPUFilterMode_Linear => FilterMode::Linear,
        _ => FilterMode::Nearest,
    }
}

fn map_mipmap_filter_mode(mode: crate::WGPUMipmapFilterMode) -> MipmapFilterMode {
    match mode {
        WGPUMipmapFilterMode_Linear => MipmapFilterMode::Linear,
        _ => MipmapFilterMode::Nearest,
    }
}

fn map_compare_function(func: crate::WGPUCompareFunction) -> Option<CompareFunction> {
    match func {
        WGPUCompareFunction_Never => Some(CompareFunction::Never),
        WGPUCompareFunction_Less => Some(CompareFunction::Less),
        WGPUCompareFunction_Equal => Some(CompareFunction::Equal),
        WGPUCompareFunction_LessEqual => Some(CompareFunction::LessEqual),
        WGPUCompareFunction_Greater => Some(CompareFunction::Greater),
        WGPUCompareFunction_NotEqual => Some(CompareFunction::NotEqual),
        WGPUCompareFunction_GreaterEqual => Some(CompareFunction::GreaterEqual),
        WGPUCompareFunction_Always => Some(CompareFunction::Always),
        _ => None,
    }
}

pub fn command_encoder_descriptor<'a>(
    d: &'a WGPUCommandEncoderDescriptor,
) -> CommandEncoderDescriptor<'a> {
    CommandEncoderDescriptor { label: unsafe { string_view(d.label) } }
}

fn map_buffer_binding_type(ty: crate::WGPUBufferBindingType) -> BufferBindingType {
    match ty {
        crate::WGPUBufferBindingType_Undefined | crate::WGPUBufferBindingType_Uniform => {
            BufferBindingType::Uniform
        }
        crate::WGPUBufferBindingType_Storage => BufferBindingType::Storage { read_only: false },
        crate::WGPUBufferBindingType_ReadOnlyStorage => {
            BufferBindingType::Storage { read_only: true }
        }
        _ => panic!("Unsupported WGPUBufferBindingType value"),
    }
}

fn map_sampler_binding_type(ty: crate::WGPUSamplerBindingType) -> SamplerBindingType {
    match ty {
        crate::WGPUSamplerBindingType_Undefined | crate::WGPUSamplerBindingType_Filtering => {
            SamplerBindingType::Filtering
        }
        crate::WGPUSamplerBindingType_NonFiltering => SamplerBindingType::NonFiltering,
        crate::WGPUSamplerBindingType_Comparison => SamplerBindingType::Comparison,
        _ => panic!("Unsupported WGPUSamplerBindingType value"),
    }
}

fn map_texture_sample_type(sample_type: crate::WGPUTextureSampleType) -> TextureSampleType {
    match sample_type {
        crate::WGPUTextureSampleType_Undefined | crate::WGPUTextureSampleType_Float => {
            TextureSampleType::Float { filterable: true }
        }
        crate::WGPUTextureSampleType_UnfilterableFloat => {
            TextureSampleType::Float { filterable: false }
        }
        crate::WGPUTextureSampleType_Depth => TextureSampleType::Depth,
        crate::WGPUTextureSampleType_Sint => TextureSampleType::Sint,
        crate::WGPUTextureSampleType_Uint => TextureSampleType::Uint,
        _ => panic!("Unsupported WGPUTextureSampleType value"),
    }
}

fn map_storage_texture_access(access: crate::WGPUStorageTextureAccess) -> StorageTextureAccess {
    match access {
        crate::WGPUStorageTextureAccess_Undefined | crate::WGPUStorageTextureAccess_WriteOnly => {
            StorageTextureAccess::WriteOnly
        }
        crate::WGPUStorageTextureAccess_ReadOnly => StorageTextureAccess::ReadOnly,
        crate::WGPUStorageTextureAccess_ReadWrite => StorageTextureAccess::ReadWrite,
        _ => panic!("Unsupported WGPUStorageTextureAccess value"),
    }
}

fn map_texture_view_dimension_for_binding(
    dimension: crate::WGPUTextureViewDimension,
    allow_cube: bool,
) -> TextureViewDimension {
    match dimension {
        crate::WGPUTextureViewDimension_Undefined | crate::WGPUTextureViewDimension_2D => {
            TextureViewDimension::D2
        }
        crate::WGPUTextureViewDimension_1D => TextureViewDimension::D1,
        crate::WGPUTextureViewDimension_2DArray => TextureViewDimension::D2Array,
        crate::WGPUTextureViewDimension_3D => TextureViewDimension::D3,
        crate::WGPUTextureViewDimension_Cube if allow_cube => TextureViewDimension::Cube,
        crate::WGPUTextureViewDimension_CubeArray if allow_cube => TextureViewDimension::CubeArray,
        _ if allow_cube => panic!("Unsupported WGPUTextureViewDimension value"),
        _ => panic!("Unsupported storage texture view dimension (cube dimensions are invalid)"),
    }
}

pub struct ConvertedBindGroupLayoutDescriptor<'a> {
    pub label: Option<&'a str>,
    pub entries: Vec<BindGroupLayoutEntry>,
}

pub fn bind_group_layout_descriptor<'a>(
    d: &'a crate::WGPUBindGroupLayoutDescriptor,
) -> ConvertedBindGroupLayoutDescriptor<'a> {
    if !d.nextInChain.is_null() {
        panic!("WGPUBindGroupLayoutDescriptor.nextInChain is not implemented");
    }
    if d.entryCount > 0 && d.entries.is_null() {
        panic!("WGPUBindGroupLayoutDescriptor.entries must not be null when entryCount > 0");
    }

    let c_entries: &[crate::WGPUBindGroupLayoutEntry] = if d.entryCount == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(d.entries, d.entryCount) }
    };

    let mut entries = Vec::with_capacity(c_entries.len());
    for e in c_entries {
        if !e.nextInChain.is_null() {
            panic!("WGPUBindGroupLayoutEntry.nextInChain is not implemented");
        }
        if !e.buffer.nextInChain.is_null()
            || !e.sampler.nextInChain.is_null()
            || !e.texture.nextInChain.is_null()
            || !e.storageTexture.nextInChain.is_null()
        {
            panic!("Chained binding layout extension structs are not implemented");
        }

        let buffer_active = e.buffer.type_ != crate::WGPUBufferBindingType_BindingNotUsed;
        let sampler_active = e.sampler.type_ != crate::WGPUSamplerBindingType_BindingNotUsed;
        let texture_active = e.texture.sampleType != crate::WGPUTextureSampleType_BindingNotUsed;
        let storage_texture_active =
            e.storageTexture.access != crate::WGPUStorageTextureAccess_BindingNotUsed;

        let active_count = u32::from(buffer_active)
            + u32::from(sampler_active)
            + u32::from(texture_active)
            + u32::from(storage_texture_active);
        if active_count != 1 {
            panic!(
                "Exactly one binding type must be active per WGPUBindGroupLayoutEntry (binding {})",
                e.binding
            );
        }

        let ty = if buffer_active {
            BindingType::Buffer {
                ty: map_buffer_binding_type(e.buffer.type_),
                has_dynamic_offset: e.buffer.hasDynamicOffset != 0,
                min_binding_size: NonZeroU64::new(e.buffer.minBindingSize),
            }
        } else if sampler_active {
            BindingType::Sampler(map_sampler_binding_type(e.sampler.type_))
        } else if texture_active {
            BindingType::Texture {
                sample_type: map_texture_sample_type(e.texture.sampleType),
                view_dimension: map_texture_view_dimension_for_binding(
                    e.texture.viewDimension,
                    true,
                ),
                multisampled: e.texture.multisampled != 0,
            }
        } else {
            BindingType::StorageTexture {
                access: map_storage_texture_access(e.storageTexture.access),
                format: map_texture_format(e.storageTexture.format),
                view_dimension: map_texture_view_dimension_for_binding(
                    e.storageTexture.viewDimension,
                    false,
                ),
            }
        };

        let count =
            if e.bindingArraySize == 0 { None } else { NonZeroU32::new(e.bindingArraySize) };

        entries.push(BindGroupLayoutEntry {
            binding: e.binding,
            visibility: ShaderStages::from_bits_truncate(e.visibility as u32),
            ty,
            count,
        });
    }

    ConvertedBindGroupLayoutDescriptor { label: unsafe { string_view(d.label) }, entries }
}

pub fn buffer_descriptor<'a>(d: &'a WGPUBufferDescriptor) -> BufferDescriptor<'a> {
    BufferDescriptor {
        label: unsafe { string_view(d.label) },
        size: d.size,
        usage: BufferUsages::from_bits_truncate(d.usage as u32),
        mapped_at_creation: d.mappedAtCreation != 0,
    }
}

pub fn texture_descriptor<'a>(d: &'a WGPUTextureDescriptor) -> TextureDescriptor<'a> {
    TextureDescriptor {
        label: unsafe { string_view(d.label) },
        size: Extent3d {
            width: d.size.width,
            height: d.size.height,
            depth_or_array_layers: d.size.depthOrArrayLayers,
        },
        mip_level_count: d.mipLevelCount,
        sample_count: d.sampleCount,
        dimension: map_texture_dimension(d.dimension),
        format: map_texture_format(d.format),
        usage: TextureUsages::from_bits_truncate(d.usage as u32),
        view_formats: &[],
    }
}

fn map_texture_dimension(dimension: crate::WGPUTextureDimension) -> TextureDimension {
    match dimension {
        WGPUTextureDimension_1D => TextureDimension::D1,
        WGPUTextureDimension_3D => TextureDimension::D3,
        _ => TextureDimension::D2,
    }
}

pub fn map_texture_format(format: crate::WGPUTextureFormat) -> TextureFormat {
    match format {
        WGPUTextureFormat_R8Unorm => TextureFormat::R8Unorm,
        WGPUTextureFormat_R8Snorm => TextureFormat::R8Snorm,
        WGPUTextureFormat_R8Uint => TextureFormat::R8Uint,
        WGPUTextureFormat_R8Sint => TextureFormat::R8Sint,
        WGPUTextureFormat_R16Unorm => TextureFormat::R16Unorm,
        WGPUTextureFormat_R16Snorm => TextureFormat::R16Snorm,
        WGPUTextureFormat_R16Uint => TextureFormat::R16Uint,
        WGPUTextureFormat_R16Sint => TextureFormat::R16Sint,
        WGPUTextureFormat_R16Float => TextureFormat::R16Float,
        WGPUTextureFormat_RG8Unorm => TextureFormat::Rg8Unorm,
        WGPUTextureFormat_RG8Snorm => TextureFormat::Rg8Snorm,
        WGPUTextureFormat_RG8Uint => TextureFormat::Rg8Uint,
        WGPUTextureFormat_RG8Sint => TextureFormat::Rg8Sint,
        WGPUTextureFormat_R32Float => TextureFormat::R32Float,
        WGPUTextureFormat_R32Uint => TextureFormat::R32Uint,
        WGPUTextureFormat_R32Sint => TextureFormat::R32Sint,
        WGPUTextureFormat_RG16Unorm => TextureFormat::Rg16Unorm,
        WGPUTextureFormat_RG16Snorm => TextureFormat::Rg16Snorm,
        WGPUTextureFormat_RG16Uint => TextureFormat::Rg16Uint,
        WGPUTextureFormat_RG16Sint => TextureFormat::Rg16Sint,
        WGPUTextureFormat_RG16Float => TextureFormat::Rg16Float,
        WGPUTextureFormat_RGBA8Unorm => TextureFormat::Rgba8Unorm,
        WGPUTextureFormat_RGBA8UnormSrgb => TextureFormat::Rgba8UnormSrgb,
        WGPUTextureFormat_RGBA8Snorm => TextureFormat::Rgba8Snorm,
        WGPUTextureFormat_RGBA8Uint => TextureFormat::Rgba8Uint,
        WGPUTextureFormat_RGBA8Sint => TextureFormat::Rgba8Sint,
        WGPUTextureFormat_BGRA8Unorm => TextureFormat::Bgra8Unorm,
        WGPUTextureFormat_BGRA8UnormSrgb => TextureFormat::Bgra8UnormSrgb,
        WGPUTextureFormat_RGB10A2Uint => TextureFormat::Rgb10a2Uint,
        WGPUTextureFormat_RGB10A2Unorm => TextureFormat::Rgb10a2Unorm,
        WGPUTextureFormat_RG11B10Ufloat => TextureFormat::Rg11b10Ufloat,
        WGPUTextureFormat_RGB9E5Ufloat => TextureFormat::Rgb9e5Ufloat,
        WGPUTextureFormat_RG32Float => TextureFormat::Rg32Float,
        WGPUTextureFormat_RG32Uint => TextureFormat::Rg32Uint,
        WGPUTextureFormat_RG32Sint => TextureFormat::Rg32Sint,
        WGPUTextureFormat_RGBA16Unorm => TextureFormat::Rgba16Unorm,
        WGPUTextureFormat_RGBA16Snorm => TextureFormat::Rgba16Snorm,
        WGPUTextureFormat_RGBA16Uint => TextureFormat::Rgba16Uint,
        WGPUTextureFormat_RGBA16Sint => TextureFormat::Rgba16Sint,
        WGPUTextureFormat_RGBA16Float => TextureFormat::Rgba16Float,
        WGPUTextureFormat_RGBA32Float => TextureFormat::Rgba32Float,
        WGPUTextureFormat_RGBA32Uint => TextureFormat::Rgba32Uint,
        WGPUTextureFormat_RGBA32Sint => TextureFormat::Rgba32Sint,
        WGPUTextureFormat_Stencil8 => TextureFormat::Stencil8,
        WGPUTextureFormat_Depth16Unorm => TextureFormat::Depth16Unorm,
        WGPUTextureFormat_Depth24Plus => TextureFormat::Depth24Plus,
        WGPUTextureFormat_Depth24PlusStencil8 => TextureFormat::Depth24PlusStencil8,
        WGPUTextureFormat_Depth32Float => TextureFormat::Depth32Float,
        WGPUTextureFormat_Depth32FloatStencil8 => TextureFormat::Depth32FloatStencil8,
        _ => panic!("Unsupported texture format value: {format}"),
    }
}

pub fn texture_view_descriptor<'a>(d: &'a WGPUTextureViewDescriptor) -> TextureViewDescriptor<'a> {
    TextureViewDescriptor {
        label: unsafe { string_view(d.label) },
        format: if d.format == WGPUTextureFormat_Undefined {
            None
        } else {
            Some(map_texture_format(d.format))
        },
        dimension: map_texture_view_dimension(d.dimension),
        usage: if d.usage == WGPUTextureUsage_None {
            None
        } else {
            Some(TextureUsages::from_bits_truncate(d.usage as u32))
        },
        aspect: map_texture_aspect(d.aspect),
        base_mip_level: d.baseMipLevel,
        mip_level_count: if d.mipLevelCount == WGPU_MIP_LEVEL_COUNT_UNDEFINED {
            None
        } else {
            Some(d.mipLevelCount)
        },
        base_array_layer: d.baseArrayLayer,
        array_layer_count: if d.arrayLayerCount == WGPU_ARRAY_LAYER_COUNT_UNDEFINED {
            None
        } else {
            Some(d.arrayLayerCount)
        },
    }
}

fn map_texture_view_dimension(
    dimension: crate::WGPUTextureViewDimension,
) -> Option<TextureViewDimension> {
    match dimension {
        WGPUTextureViewDimension_Undefined => None,
        WGPUTextureViewDimension_1D => Some(TextureViewDimension::D1),
        WGPUTextureViewDimension_2D => Some(TextureViewDimension::D2),
        WGPUTextureViewDimension_2DArray => Some(TextureViewDimension::D2Array),
        WGPUTextureViewDimension_Cube => Some(TextureViewDimension::Cube),
        WGPUTextureViewDimension_CubeArray => Some(TextureViewDimension::CubeArray),
        WGPUTextureViewDimension_3D => Some(TextureViewDimension::D3),
        _ => None,
    }
}

fn map_texture_aspect(aspect: crate::WGPUTextureAspect) -> TextureAspect {
    match aspect {
        WGPUTextureAspect_StencilOnly => TextureAspect::StencilOnly,
        WGPUTextureAspect_DepthOnly => TextureAspect::DepthOnly,
        WGPUTextureAspect_Undefined => TextureAspect::All,
        _ => TextureAspect::All,
    }
}
