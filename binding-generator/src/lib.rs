/*
 * Copyright 2024 wgpu-native contributors
 * Copyright 2026 MapLibre contributors
 *
 * This file contains code copied from wgpu-native (https://github.com/gfx-rs/wgpu-native)
 * Licensed under the Apache License, Version 2.0 or the MIT License, at your option.
 */

#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_qualifications)]
#![allow(unused_variables)]
#![allow(clippy::all)]
#![allow(clippy::missing_safety_doc)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use wgpu::{SamplerDescriptor, SubmissionIndex};

mod conv;

fn mapped_buffer_views() -> &'static Mutex<HashMap<usize, Vec<wgpu::BufferViewMut>>> {
    static VIEWS: OnceLock<Mutex<HashMap<usize, Vec<wgpu::BufferViewMut>>>> = OnceLock::new();
    VIEWS.get_or_init(|| Mutex::new(HashMap::new()))
}

macro_rules! opaque_handle_types {
	($($name:ident),+ $(,)?) => {
		$(
			#[repr(C)]
			#[derive(Debug)]
			pub struct $name {
				_unused: [u8; 0],
			}
		)+
	};
}

pub struct WGPUDeviceImpl(wgpu::Device);
pub struct WGPUQueueImpl(wgpu::Queue);

impl WGPUDeviceImpl {
    pub fn to_pointer(self) -> WGPUDevice {
        Arc::into_raw(Arc::new(self))
    }
}

impl WGPUQueueImpl {
    pub fn to_pointer(self) -> WGPUQueue {
        Arc::into_raw(Arc::new(self))
    }
}

pub struct WGPUSamplerImpl(wgpu::Sampler);

impl WGPUSamplerImpl {
    pub fn to_pointer(self) -> WGPUSampler {
        Arc::into_raw(Arc::new(self))
    }
}

pub struct WGPUCommandEncoderImpl(Mutex<Option<wgpu::CommandEncoder>>);

impl WGPUCommandEncoderImpl {
    pub fn to_pointer(self) -> WGPUCommandEncoder {
        Arc::into_raw(Arc::new(self))
    }
}

pub struct WGPUCommandBufferImpl(wgpu::CommandBuffer);

impl WGPUCommandBufferImpl {
    pub fn to_pointer(self) -> WGPUCommandBuffer {
        Arc::into_raw(Arc::new(self))
    }
}

pub struct WGPURenderPassEncoderImpl(Mutex<Option<wgpu::RenderPass<'static>>>);

impl WGPURenderPassEncoderImpl {
    pub fn to_pointer(self) -> WGPURenderPassEncoder {
        Arc::into_raw(Arc::new(self))
    }
}

pub struct WGPUBufferImpl(wgpu::Buffer);

impl WGPUBufferImpl {
    pub fn to_pointer(self) -> WGPUBuffer {
        Arc::into_raw(Arc::new(self))
    }
}

pub struct WGPUTextureImpl(wgpu::Texture);

impl WGPUTextureImpl {
    pub fn to_pointer(self) -> WGPUTexture {
        Arc::into_raw(Arc::new(self))
    }
}

pub struct WGPUTextureViewImpl(wgpu::TextureView);

impl WGPUTextureViewImpl {
    pub fn to_pointer(self) -> WGPUTextureView {
        Arc::into_raw(Arc::new(self))
    }
}

pub struct WGPURenderPipelineImpl(wgpu::RenderPipeline);

impl WGPURenderPipelineImpl {
    pub fn to_pointer(self) -> WGPURenderPipeline {
        Arc::into_raw(Arc::new(self))
    }
}

pub struct WGPUBindGroupLayoutImpl(wgpu::BindGroupLayout);

impl WGPUBindGroupLayoutImpl {
    pub fn to_pointer(self) -> WGPUBindGroupLayout {
        Arc::into_raw(Arc::new(self))
    }
}

pub struct WGPUDeviceWrapper(WGPUDevice);
pub struct WGPUQueueWrapper(WGPUQueue);

impl From<wgpu::Device> for WGPUDeviceWrapper {
    fn from(value: wgpu::Device) -> Self {
        let pointer = WGPUDeviceImpl(value).to_pointer();
        Self(pointer)
    }
}

impl From<wgpu::Queue> for WGPUQueueWrapper {
    fn from(value: wgpu::Queue) -> Self {
        Self(WGPUQueueImpl(value).to_pointer())
    }
}

unsafe impl cxx::ExternType for WGPUDeviceWrapper {
    type Id = cxx::type_id!("WGPUDevice");
    type Kind = cxx::kind::Trivial;
}

unsafe impl cxx::ExternType for WGPUQueueWrapper {
    type Id = cxx::type_id!("WGPUQueue");
    type Kind = cxx::kind::Trivial;
}

opaque_handle_types!(
    WGPUAdapterImpl,
    WGPUBindGroupImpl,
    WGPUComputePassEncoderImpl,
    WGPUComputePipelineImpl,
    WGPUInstanceImpl,
    WGPUPipelineLayoutImpl,
    WGPUQuerySetImpl,
    WGPURenderBundleImpl,
    WGPURenderBundleEncoderImpl,
    WGPUShaderModuleImpl,
    WGPUSurfaceImpl,
);

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCreateInstance(
    descriptor: *const WGPUInstanceDescriptor,
) -> WGPUInstance {
    panic!("wgpuCreateInstance must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuGetInstanceFeatures(features: *mut WGPUSupportedInstanceFeatures) {
    panic!("wgpuGetInstanceFeatures must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuGetInstanceLimits(limits: *mut WGPUInstanceLimits) -> WGPUStatus {
    panic!("wgpuGetInstanceLimits must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuHasInstanceFeature(feature: WGPUInstanceFeatureName) -> WGPUBool {
    panic!("wgpuHasInstanceFeature must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuGetProcAddress(procName: WGPUStringView) -> WGPUProc {
    panic!("wgpuGetProcAddress must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuAdapterGetFeatures(
    adapter: WGPUAdapter,
    features: *mut WGPUSupportedFeatures,
) {
    panic!("wgpuAdapterGetFeatures must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuAdapterGetInfo(
    adapter: WGPUAdapter,
    info: *mut WGPUAdapterInfo,
) -> WGPUStatus {
    panic!("wgpuAdapterGetInfo must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuAdapterGetLimits(
    adapter: WGPUAdapter,
    limits: *mut WGPULimits,
) -> WGPUStatus {
    panic!("wgpuAdapterGetLimits must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuAdapterHasFeature(
    adapter: WGPUAdapter,
    feature: WGPUFeatureName,
) -> WGPUBool {
    panic!("wgpuAdapterHasFeature must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuAdapterRequestDevice(
    adapter: WGPUAdapter,
    descriptor: *const WGPUDeviceDescriptor,
    callbackInfo: WGPURequestDeviceCallbackInfo,
) -> WGPUFuture {
    panic!("wgpuAdapterRequestDevice must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuAdapterAddRef(adapter: WGPUAdapter) {
    panic!("wgpuAdapterAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuAdapterRelease(adapter: WGPUAdapter) {
    panic!("wgpuAdapterRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuAdapterInfoFreeMembers(adapterInfo: WGPUAdapterInfo) {
    panic!("wgpuAdapterInfoFreeMembers must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBindGroupSetLabel(bindGroup: WGPUBindGroup, label: WGPUStringView) {
    panic!("wgpuBindGroupSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBindGroupAddRef(bindGroup: WGPUBindGroup) {
    panic!("wgpuBindGroupAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBindGroupRelease(bindGroup: WGPUBindGroup) {
    panic!("wgpuBindGroupRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBindGroupLayoutSetLabel(
    bindGroupLayout: WGPUBindGroupLayout,
    label: WGPUStringView,
) {
    panic!("wgpuBindGroupLayoutSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBindGroupLayoutAddRef(bindGroupLayout: WGPUBindGroupLayout) {
    panic!("wgpuBindGroupLayoutAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBindGroupLayoutRelease(bindGroupLayout: WGPUBindGroupLayout) {
    panic!("wgpuBindGroupLayoutRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferDestroy(buffer: WGPUBuffer) {
    panic!("wgpuBufferDestroy must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferGetConstMappedRange(
    buffer: WGPUBuffer,
    offset: usize,
    size: usize,
) -> *const ::std::os::raw::c_void {
    panic!("wgpuBufferGetConstMappedRange must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferGetMappedRange(
    buffer: WGPUBuffer,
    offset: usize,
    size: usize,
) -> *mut ::std::os::raw::c_void {
    let buffer_ref = unsafe { buffer.as_ref().expect("Invalid buffer") };
    let offset_u64 = u64::try_from(offset).expect("offset does not fit in u64");

    let mut view = if size == usize::MAX {
        buffer_ref.0.slice(offset_u64..).get_mapped_range_mut()
    } else {
        let size_u64 = u64::try_from(size).expect("size does not fit in u64");
        let end = offset_u64.checked_add(size_u64).expect("offset + size overflow");
        buffer_ref.0.slice(offset_u64..end).get_mapped_range_mut()
    };

    let ptr = view.slice(..).as_raw_element_ptr().as_ptr().cast();
    mapped_buffer_views()
        .lock()
        .expect("mapped buffer registry lock poisoned")
        .entry(buffer as usize)
        .or_default()
        .push(view);
    ptr
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferGetMapState(buffer: WGPUBuffer) -> WGPUBufferMapState {
    panic!("wgpuBufferGetMapState must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferGetSize(buffer: WGPUBuffer) -> u64 {
    panic!("wgpuBufferGetSize must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferGetUsage(buffer: WGPUBuffer) -> WGPUBufferUsage {
    panic!("wgpuBufferGetUsage must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferMapAsync(
    buffer: WGPUBuffer,
    mode: WGPUMapMode,
    offset: usize,
    size: usize,
    callbackInfo: WGPUBufferMapCallbackInfo,
) -> WGPUFuture {
    panic!("wgpuBufferMapAsync must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferReadMappedRange(
    buffer: WGPUBuffer,
    offset: usize,
    data: *mut ::std::os::raw::c_void,
    size: usize,
) -> WGPUStatus {
    panic!("wgpuBufferReadMappedRange must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferSetLabel(buffer: WGPUBuffer, label: WGPUStringView) {
    panic!("wgpuBufferSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferUnmap(buffer: WGPUBuffer) {
    let buffer_ref = unsafe { buffer.as_ref().expect("Invalid buffer") };
    let mut views = mapped_buffer_views().lock().expect("mapped buffer registry lock poisoned");
    views.remove(&(buffer as usize));
    drop(views);
    buffer_ref.0.unmap();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferWriteMappedRange(
    buffer: WGPUBuffer,
    offset: usize,
    data: *const ::std::os::raw::c_void,
    size: usize,
) -> WGPUStatus {
    panic!("wgpuBufferWriteMappedRange must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferAddRef(buffer: WGPUBuffer) {
    panic!("wgpuBufferAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuBufferRelease(buffer: WGPUBuffer) {
    panic!("wgpuBufferRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandBufferSetLabel(
    commandBuffer: WGPUCommandBuffer,
    label: WGPUStringView,
) {
    panic!("wgpuCommandBufferSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandBufferAddRef(commandBuffer: WGPUCommandBuffer) {
    panic!("wgpuCommandBufferAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandBufferRelease(commandBuffer: WGPUCommandBuffer) {
    let _ = unsafe { commandBuffer.as_ref().expect("Invalid commandBuffer") };
    unsafe {
        drop(Arc::from_raw(commandBuffer));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderBeginComputePass(
    commandEncoder: WGPUCommandEncoder,
    descriptor: *const WGPUComputePassDescriptor,
) -> WGPUComputePassEncoder {
    panic!("wgpuCommandEncoderBeginComputePass must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderBeginRenderPass(
    commandEncoder: WGPUCommandEncoder,
    descriptor: *const WGPURenderPassDescriptor,
) -> WGPURenderPassEncoder {
    let encoder_ref = unsafe { commandEncoder.as_ref().expect("Invalid commandEncoder") };
    let desc = unsafe { descriptor.as_ref().expect("WGPURenderPassDescriptor must not be null") };

    if !desc.timestampWrites.is_null() {
        panic!("wgpuCommandEncoderBeginRenderPass timestampWrites not implemented");
    }
    if !desc.occlusionQuerySet.is_null() {
        panic!("wgpuCommandEncoderBeginRenderPass occlusionQuerySet not implemented");
    }

    let mut color_attachments = Vec::with_capacity(desc.colorAttachmentCount);
    if desc.colorAttachmentCount > 0 && !desc.colorAttachments.is_null() {
        let c_color_attachments =
            unsafe { std::slice::from_raw_parts(desc.colorAttachments, desc.colorAttachmentCount) };
        for c in c_color_attachments {
            if c.view.is_null() {
                color_attachments.push(None);
                continue;
            }
            let view = unsafe { c.view.as_ref().expect("Invalid color attachment view") };
            let resolve_target = if c.resolveTarget.is_null() {
                None
            } else {
                Some(
                    &unsafe {
                        c.resolveTarget.as_ref().expect("Invalid resolve target texture view")
                    }
                    .0,
                )
            };
            let load = match c.loadOp {
                WGPULoadOp_Clear => wgpu::LoadOp::Clear(wgpu::Color {
                    r: c.clearValue.r,
                    g: c.clearValue.g,
                    b: c.clearValue.b,
                    a: c.clearValue.a,
                }),
                _ => wgpu::LoadOp::Load,
            };
            let store = match c.storeOp {
                WGPUStoreOp_Discard => wgpu::StoreOp::Discard,
                _ => wgpu::StoreOp::Store,
            };
            let depth_slice = if c.depthSlice == u32::MAX { None } else { Some(c.depthSlice) };
            color_attachments.push(Some(wgpu::RenderPassColorAttachment {
                view: &view.0,
                depth_slice,
                resolve_target,
                ops: wgpu::Operations { load, store },
            }));
        }
    }

    let depth_stencil_attachment = if desc.depthStencilAttachment.is_null() {
        None
    } else {
        let c = unsafe {
            desc.depthStencilAttachment.as_ref().expect("Invalid depth stencil attachment")
        };
        let view = unsafe { c.view.as_ref().expect("Invalid depth stencil attachment view") };
        let depth_ops = if c.depthReadOnly != 0 {
            None
        } else {
            Some(wgpu::Operations {
                load: match c.depthLoadOp {
                    WGPULoadOp_Clear => wgpu::LoadOp::Clear(c.depthClearValue),
                    _ => wgpu::LoadOp::Load,
                },
                store: match c.depthStoreOp {
                    WGPUStoreOp_Discard => wgpu::StoreOp::Discard,
                    _ => wgpu::StoreOp::Store,
                },
            })
        };
        let stencil_ops = if c.stencilReadOnly != 0 {
            None
        } else {
            Some(wgpu::Operations {
                load: match c.stencilLoadOp {
                    WGPULoadOp_Clear => wgpu::LoadOp::Clear(c.stencilClearValue),
                    _ => wgpu::LoadOp::Load,
                },
                store: match c.stencilStoreOp {
                    WGPUStoreOp_Discard => wgpu::StoreOp::Discard,
                    _ => wgpu::StoreOp::Store,
                },
            })
        };
        Some(wgpu::RenderPassDepthStencilAttachment { view: &view.0, depth_ops, stencil_ops })
    };

    let render_pass_desc = wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: color_attachments.as_slice(),
        depth_stencil_attachment,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    };

    let mut encoder_guard = encoder_ref.0.lock().expect("command encoder lock poisoned");
    let render_pass = encoder_guard
        .as_mut()
        .expect("command encoder already finished")
        .begin_render_pass(&render_pass_desc)
        .forget_lifetime();
    WGPURenderPassEncoderImpl(Mutex::new(Some(render_pass))).to_pointer()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderClearBuffer(
    commandEncoder: WGPUCommandEncoder,
    buffer: WGPUBuffer,
    offset: u64,
    size: u64,
) {
    panic!("wgpuCommandEncoderClearBuffer must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderCopyBufferToBuffer(
    commandEncoder: WGPUCommandEncoder,
    source: WGPUBuffer,
    sourceOffset: u64,
    destination: WGPUBuffer,
    destinationOffset: u64,
    size: u64,
) {
    panic!("wgpuCommandEncoderCopyBufferToBuffer must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderCopyBufferToTexture(
    commandEncoder: WGPUCommandEncoder,
    source: *const WGPUTexelCopyBufferInfo,
    destination: *const WGPUTexelCopyTextureInfo,
    copySize: *const WGPUExtent3D,
) {
    panic!("wgpuCommandEncoderCopyBufferToTexture must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderCopyTextureToBuffer(
    commandEncoder: WGPUCommandEncoder,
    source: *const WGPUTexelCopyTextureInfo,
    destination: *const WGPUTexelCopyBufferInfo,
    copySize: *const WGPUExtent3D,
) {
    panic!("wgpuCommandEncoderCopyTextureToBuffer must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderCopyTextureToTexture(
    commandEncoder: WGPUCommandEncoder,
    source: *const WGPUTexelCopyTextureInfo,
    destination: *const WGPUTexelCopyTextureInfo,
    copySize: *const WGPUExtent3D,
) {
    panic!("wgpuCommandEncoderCopyTextureToTexture must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderFinish(
    commandEncoder: WGPUCommandEncoder,
    descriptor: *const WGPUCommandBufferDescriptor,
) -> WGPUCommandBuffer {
    let encoder_ref = unsafe { commandEncoder.as_ref().expect("Invalid commandEncoder") };
    let encoder = encoder_ref
        .0
        .lock()
        .expect("command encoder lock poisoned")
        .take()
        .expect("command encoder already finished");
    let cmd_buf = encoder.finish();
    WGPUCommandBufferImpl(cmd_buf).to_pointer()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderInsertDebugMarker(
    commandEncoder: WGPUCommandEncoder,
    markerLabel: WGPUStringView,
) {
    panic!("wgpuCommandEncoderInsertDebugMarker must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderPopDebugGroup(commandEncoder: WGPUCommandEncoder) {
    panic!("wgpuCommandEncoderPopDebugGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderPushDebugGroup(
    commandEncoder: WGPUCommandEncoder,
    groupLabel: WGPUStringView,
) {
    panic!("wgpuCommandEncoderPushDebugGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderResolveQuerySet(
    commandEncoder: WGPUCommandEncoder,
    querySet: WGPUQuerySet,
    firstQuery: u32,
    queryCount: u32,
    destination: WGPUBuffer,
    destinationOffset: u64,
) {
    panic!("wgpuCommandEncoderResolveQuerySet must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderSetLabel(
    commandEncoder: WGPUCommandEncoder,
    label: WGPUStringView,
) {
    panic!("wgpuCommandEncoderSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderWriteTimestamp(
    commandEncoder: WGPUCommandEncoder,
    querySet: WGPUQuerySet,
    queryIndex: u32,
) {
    panic!("wgpuCommandEncoderWriteTimestamp must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderAddRef(commandEncoder: WGPUCommandEncoder) {
    panic!("wgpuCommandEncoderAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuCommandEncoderRelease(commandEncoder: WGPUCommandEncoder) {
    let _ = unsafe { commandEncoder.as_ref().expect("Invalid commandEncoder") };
    unsafe {
        drop(Arc::from_raw(commandEncoder));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePassEncoderDispatchWorkgroups(
    computePassEncoder: WGPUComputePassEncoder,
    workgroupCountX: u32,
    workgroupCountY: u32,
    workgroupCountZ: u32,
) {
    panic!("wgpuComputePassEncoderDispatchWorkgroups must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePassEncoderDispatchWorkgroupsIndirect(
    computePassEncoder: WGPUComputePassEncoder,
    indirectBuffer: WGPUBuffer,
    indirectOffset: u64,
) {
    panic!("wgpuComputePassEncoderDispatchWorkgroupsIndirect must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePassEncoderEnd(computePassEncoder: WGPUComputePassEncoder) {
    panic!("wgpuComputePassEncoderEnd must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePassEncoderInsertDebugMarker(
    computePassEncoder: WGPUComputePassEncoder,
    markerLabel: WGPUStringView,
) {
    panic!("wgpuComputePassEncoderInsertDebugMarker must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePassEncoderPopDebugGroup(
    computePassEncoder: WGPUComputePassEncoder,
) {
    panic!("wgpuComputePassEncoderPopDebugGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePassEncoderPushDebugGroup(
    computePassEncoder: WGPUComputePassEncoder,
    groupLabel: WGPUStringView,
) {
    panic!("wgpuComputePassEncoderPushDebugGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePassEncoderSetBindGroup(
    computePassEncoder: WGPUComputePassEncoder,
    groupIndex: u32,
    group: WGPUBindGroup,
    dynamicOffsetCount: usize,
    dynamicOffsets: *const u32,
) {
    panic!("wgpuComputePassEncoderSetBindGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePassEncoderSetLabel(
    computePassEncoder: WGPUComputePassEncoder,
    label: WGPUStringView,
) {
    panic!("wgpuComputePassEncoderSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePassEncoderSetPipeline(
    computePassEncoder: WGPUComputePassEncoder,
    pipeline: WGPUComputePipeline,
) {
    panic!("wgpuComputePassEncoderSetPipeline must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePassEncoderAddRef(computePassEncoder: WGPUComputePassEncoder) {
    panic!("wgpuComputePassEncoderAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePassEncoderRelease(computePassEncoder: WGPUComputePassEncoder) {
    panic!("wgpuComputePassEncoderRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePipelineGetBindGroupLayout(
    computePipeline: WGPUComputePipeline,
    groupIndex: u32,
) -> WGPUBindGroupLayout {
    panic!("wgpuComputePipelineGetBindGroupLayout must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePipelineSetLabel(
    computePipeline: WGPUComputePipeline,
    label: WGPUStringView,
) {
    panic!("wgpuComputePipelineSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePipelineAddRef(computePipeline: WGPUComputePipeline) {
    panic!("wgpuComputePipelineAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuComputePipelineRelease(computePipeline: WGPUComputePipeline) {
    panic!("wgpuComputePipelineRelease must be implemented");
}

fn handle_error_fatal(
    cause: impl std::error::Error + Send + Sync + 'static,
    operation: &'static str,
) -> ! {
    panic!("Error in {operation}"); //: {f}", f = format_error(&cause));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDevicePoll(
    device: WGPUDevice,
    wait: bool,
    submission_index: Option<&WGPUSubmissionIndex>,
) -> bool {
    let device = unsafe { device.as_ref() }.expect("invalid device");

    let poll = match wait {
        true => match submission_index {
            None => wgpu::PollType::wait_indefinitely(),
            _ => panic!("Not implemented"),
        },
        false => wgpu::PollType::Poll,
    };

    match device.0.poll(poll) {
        Ok(wgpu::PollStatus::QueueEmpty) => true,
        Ok(_) => false,
        Err(cause) => {
            handle_error_fatal(cause, "wgpuDevicePoll");
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateBindGroup(
    device: WGPUDevice,
    descriptor: *const WGPUBindGroupDescriptor,
) -> WGPUBindGroup {
    panic!("wgpuDeviceCreateBindGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateBindGroupLayout(
    device: WGPUDevice,
    descriptor: *const WGPUBindGroupLayoutDescriptor,
) -> WGPUBindGroupLayout {
    panic!("wgpuDeviceCreateBindGroupLayout must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateBuffer(
    device: WGPUDevice,
    descriptor: *const WGPUBufferDescriptor,
) -> WGPUBuffer {
    let d = unsafe { descriptor.as_ref().expect("WGPUBufferDescriptor must not be null") };
    let device_ref = unsafe { device.as_ref().expect("Invalid device") };
    let buffer = device_ref.0.create_buffer(&conv::buffer_descriptor(d));
    WGPUBufferImpl(buffer).to_pointer()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateCommandEncoder(
    device: WGPUDevice,
    descriptor: *const WGPUCommandEncoderDescriptor,
) -> WGPUCommandEncoder {
    let wgpu_desc = match unsafe { descriptor.as_ref() } {
        Some(d) => conv::command_encoder_descriptor(d),
        None => wgpu::CommandEncoderDescriptor::default(),
    };
    let device_ref = unsafe { device.as_ref().expect("Invalid device") };
    let encoder = device_ref.0.create_command_encoder(&wgpu_desc);
    WGPUCommandEncoderImpl(Mutex::new(Some(encoder))).to_pointer()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateComputePipeline(
    device: WGPUDevice,
    descriptor: *const WGPUComputePipelineDescriptor,
) -> WGPUComputePipeline {
    panic!("wgpuDeviceCreateComputePipeline must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateComputePipelineAsync(
    device: WGPUDevice,
    descriptor: *const WGPUComputePipelineDescriptor,
    callbackInfo: WGPUCreateComputePipelineAsyncCallbackInfo,
) -> WGPUFuture {
    panic!("wgpuDeviceCreateComputePipelineAsync must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreatePipelineLayout(
    device: WGPUDevice,
    descriptor: *const WGPUPipelineLayoutDescriptor,
) -> WGPUPipelineLayout {
    panic!("wgpuDeviceCreatePipelineLayout must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateQuerySet(
    device: WGPUDevice,
    descriptor: *const WGPUQuerySetDescriptor,
) -> WGPUQuerySet {
    panic!("wgpuDeviceCreateQuerySet must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateRenderBundleEncoder(
    device: WGPUDevice,
    descriptor: *const WGPURenderBundleEncoderDescriptor,
) -> WGPURenderBundleEncoder {
    panic!("wgpuDeviceCreateRenderBundleEncoder must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateRenderPipeline(
    device: WGPUDevice,
    descriptor: *const WGPURenderPipelineDescriptor,
) -> WGPURenderPipeline {
    panic!("wgpuDeviceCreateRenderPipeline must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateRenderPipelineAsync(
    device: WGPUDevice,
    descriptor: *const WGPURenderPipelineDescriptor,
    callbackInfo: WGPUCreateRenderPipelineAsyncCallbackInfo,
) -> WGPUFuture {
    panic!("wgpuDeviceCreateRenderPipelineAsync must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateSampler(
    device: WGPUDevice,
    descriptor: *const WGPUSamplerDescriptor,
) -> WGPUSampler {
    let wgpu_desc = match unsafe { descriptor.as_ref() } {
        Some(d) => conv::sampler_descriptor(d),
        None => SamplerDescriptor::default(),
    };
    let device_ref = unsafe { device.as_ref().expect("Invalid device") };
    let sampler = device_ref.0.create_sampler(&wgpu_desc);
    WGPUSamplerImpl(sampler).to_pointer()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateShaderModule(
    device: WGPUDevice,
    descriptor: *const WGPUShaderModuleDescriptor,
) -> WGPUShaderModule {
    panic!("wgpuDeviceCreateShaderModule must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceCreateTexture(
    device: WGPUDevice,
    descriptor: *const WGPUTextureDescriptor,
) -> WGPUTexture {
    let d = unsafe { descriptor.as_ref().expect("WGPUTextureDescriptor must not be null") };
    let mut wgpu_desc = conv::texture_descriptor(d);
    let mut mapped_view_formats = Vec::new();
    if d.viewFormatCount > 0 && !d.viewFormats.is_null() {
        let c_view_formats =
            unsafe { std::slice::from_raw_parts(d.viewFormats, d.viewFormatCount) };
        mapped_view_formats.reserve(c_view_formats.len());
        for &format in c_view_formats {
            mapped_view_formats.push(conv::map_texture_format(format));
        }
        wgpu_desc.view_formats = mapped_view_formats.as_slice();
    }
    let device_ref = unsafe { device.as_ref().expect("Invalid device") };
    let texture = device_ref.0.create_texture(&wgpu_desc);
    WGPUTextureImpl(texture).to_pointer()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceDestroy(device: WGPUDevice) {
    unsafe {
        device.as_ref().expect("Invalid device").0.destroy();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceGetAdapterInfo(
    device: WGPUDevice,
    adapterInfo: *mut WGPUAdapterInfo,
) -> WGPUStatus {
    panic!("wgpuDeviceGetAdapterInfo must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceGetFeatures(
    device: WGPUDevice,
    features: *mut WGPUSupportedFeatures,
) {
    panic!("wgpuDeviceGetFeatures must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceGetLimits(
    device: WGPUDevice,
    limits: *mut WGPULimits,
) -> WGPUStatus {
    panic!("wgpuDeviceGetLimits must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceGetLostFuture(device: WGPUDevice) -> WGPUFuture {
    panic!("wgpuDeviceGetLostFuture must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceGetQueue(device: WGPUDevice) -> WGPUQueue {
    panic!("wgpuDeviceGetQueue must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceHasFeature(
    device: WGPUDevice,
    feature: WGPUFeatureName,
) -> WGPUBool {
    panic!("wgpuDeviceHasFeature must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDevicePopErrorScope(
    device: WGPUDevice,
    callbackInfo: WGPUPopErrorScopeCallbackInfo,
) -> WGPUFuture {
    panic!("wgpuDevicePopErrorScope must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDevicePushErrorScope(device: WGPUDevice, filter: WGPUErrorFilter) {
    panic!("wgpuDevicePushErrorScope must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceSetLabel(device: WGPUDevice, label: WGPUStringView) {
    panic!("wgpuDeviceSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceAddRef(device: WGPUDevice) {
    panic!("wgpuDeviceAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuDeviceRelease(device: WGPUDevice) {
    panic!("wgpuDeviceRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuExternalTextureSetLabel(
    externalTexture: WGPUExternalTexture,
    label: WGPUStringView,
) {
    panic!("wgpuExternalTextureSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuExternalTextureAddRef(externalTexture: WGPUExternalTexture) {
    panic!("wgpuExternalTextureAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuExternalTextureRelease(externalTexture: WGPUExternalTexture) {
    panic!("wgpuExternalTextureRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuInstanceCreateSurface(
    instance: WGPUInstance,
    descriptor: *const WGPUSurfaceDescriptor,
) -> WGPUSurface {
    panic!("wgpuInstanceCreateSurface must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuInstanceGetWGSLLanguageFeatures(
    instance: WGPUInstance,
    features: *mut WGPUSupportedWGSLLanguageFeatures,
) {
    panic!("wgpuInstanceGetWGSLLanguageFeatures must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuInstanceHasWGSLLanguageFeature(
    instance: WGPUInstance,
    feature: WGPUWGSLLanguageFeatureName,
) -> WGPUBool {
    panic!("wgpuInstanceHasWGSLLanguageFeature must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuInstanceProcessEvents(instance: WGPUInstance) {
    panic!("wgpuInstanceProcessEvents must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuInstanceRequestAdapter(
    instance: WGPUInstance,
    options: *const WGPURequestAdapterOptions,
    callbackInfo: WGPURequestAdapterCallbackInfo,
) -> WGPUFuture {
    panic!("wgpuInstanceRequestAdapter must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuInstanceWaitAny(
    instance: WGPUInstance,
    futureCount: usize,
    futures: *mut WGPUFutureWaitInfo,
    timeoutNS: u64,
) -> WGPUWaitStatus {
    panic!("wgpuInstanceWaitAny must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuInstanceAddRef(instance: WGPUInstance) {
    panic!("wgpuInstanceAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuInstanceRelease(instance: WGPUInstance) {
    panic!("wgpuInstanceRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuPipelineLayoutSetLabel(
    pipelineLayout: WGPUPipelineLayout,
    label: WGPUStringView,
) {
    panic!("wgpuPipelineLayoutSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuPipelineLayoutAddRef(pipelineLayout: WGPUPipelineLayout) {
    panic!("wgpuPipelineLayoutAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuPipelineLayoutRelease(pipelineLayout: WGPUPipelineLayout) {
    panic!("wgpuPipelineLayoutRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQuerySetDestroy(querySet: WGPUQuerySet) {
    panic!("wgpuQuerySetDestroy must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQuerySetGetCount(querySet: WGPUQuerySet) -> u32 {
    panic!("wgpuQuerySetGetCount must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQuerySetGetType(querySet: WGPUQuerySet) -> WGPUQueryType {
    panic!("wgpuQuerySetGetType must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQuerySetSetLabel(querySet: WGPUQuerySet, label: WGPUStringView) {
    panic!("wgpuQuerySetSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQuerySetAddRef(querySet: WGPUQuerySet) {
    panic!("wgpuQuerySetAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQuerySetRelease(querySet: WGPUQuerySet) {
    panic!("wgpuQuerySetRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQueueOnSubmittedWorkDone(
    queue: WGPUQueue,
    callbackInfo: WGPUQueueWorkDoneCallbackInfo,
) -> WGPUFuture {
    panic!("wgpuQueueOnSubmittedWorkDone must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQueueSetLabel(queue: WGPUQueue, label: WGPUStringView) {
    panic!("wgpuQueueSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQueueSubmit(
    queue: WGPUQueue,
    commandCount: usize,
    commands: *const WGPUCommandBuffer,
) {
    let queue_ref = unsafe { queue.as_ref().expect("Invalid queue") };
    let mut cmd_bufs = Vec::with_capacity(commandCount);
    for i in 0..commandCount {
        let ptr = unsafe { *commands.add(i) };
        let arc = unsafe { Arc::from_raw(ptr) };
        let impl_ = Arc::try_unwrap(arc)
            .unwrap_or_else(|_| panic!("CommandBuffer has extra references on submit"));
        cmd_bufs.push(impl_.0);
    }
    queue_ref.0.submit(cmd_bufs);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQueueWriteBuffer(
    queue: WGPUQueue,
    buffer: WGPUBuffer,
    bufferOffset: u64,
    data: *const ::std::os::raw::c_void,
    size: usize,
) {
    let queue_ref = unsafe { queue.as_ref().expect("Invalid queue") };
    let buffer_ref = unsafe { buffer.as_ref().expect("Invalid buffer") };
    let bytes: &[u8] = if size == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(data.cast::<u8>(), size) }
    };
    queue_ref.0.write_buffer(&buffer_ref.0, bufferOffset, bytes);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQueueWriteTexture(
    queue: WGPUQueue,
    destination: *const WGPUTexelCopyTextureInfo,
    data: *const ::std::os::raw::c_void,
    dataSize: usize,
    dataLayout: *const WGPUTexelCopyBufferLayout,
    writeSize: *const WGPUExtent3D,
) {
    panic!("wgpuQueueWriteTexture must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQueueAddRef(queue: WGPUQueue) {
    panic!("wgpuQueueAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuQueueRelease(queue: WGPUQueue) {
    panic!("wgpuQueueRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleSetLabel(
    renderBundle: WGPURenderBundle,
    label: WGPUStringView,
) {
    panic!("wgpuRenderBundleSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleAddRef(renderBundle: WGPURenderBundle) {
    panic!("wgpuRenderBundleAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleRelease(renderBundle: WGPURenderBundle) {
    panic!("wgpuRenderBundleRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderDraw(
    renderBundleEncoder: WGPURenderBundleEncoder,
    vertexCount: u32,
    instanceCount: u32,
    firstVertex: u32,
    firstInstance: u32,
) {
    panic!("wgpuRenderBundleEncoderDraw must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderDrawIndexed(
    renderBundleEncoder: WGPURenderBundleEncoder,
    indexCount: u32,
    instanceCount: u32,
    firstIndex: u32,
    baseVertex: i32,
    firstInstance: u32,
) {
    panic!("wgpuRenderBundleEncoderDrawIndexed must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderDrawIndexedIndirect(
    renderBundleEncoder: WGPURenderBundleEncoder,
    indirectBuffer: WGPUBuffer,
    indirectOffset: u64,
) {
    panic!("wgpuRenderBundleEncoderDrawIndexedIndirect must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderDrawIndirect(
    renderBundleEncoder: WGPURenderBundleEncoder,
    indirectBuffer: WGPUBuffer,
    indirectOffset: u64,
) {
    panic!("wgpuRenderBundleEncoderDrawIndirect must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderFinish(
    renderBundleEncoder: WGPURenderBundleEncoder,
    descriptor: *const WGPURenderBundleDescriptor,
) -> WGPURenderBundle {
    panic!("wgpuRenderBundleEncoderFinish must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderInsertDebugMarker(
    renderBundleEncoder: WGPURenderBundleEncoder,
    markerLabel: WGPUStringView,
) {
    panic!("wgpuRenderBundleEncoderInsertDebugMarker must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderPopDebugGroup(
    renderBundleEncoder: WGPURenderBundleEncoder,
) {
    panic!("wgpuRenderBundleEncoderPopDebugGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderPushDebugGroup(
    renderBundleEncoder: WGPURenderBundleEncoder,
    groupLabel: WGPUStringView,
) {
    panic!("wgpuRenderBundleEncoderPushDebugGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderSetBindGroup(
    renderBundleEncoder: WGPURenderBundleEncoder,
    groupIndex: u32,
    group: WGPUBindGroup,
    dynamicOffsetCount: usize,
    dynamicOffsets: *const u32,
) {
    panic!("wgpuRenderBundleEncoderSetBindGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderSetIndexBuffer(
    renderBundleEncoder: WGPURenderBundleEncoder,
    buffer: WGPUBuffer,
    format: WGPUIndexFormat,
    offset: u64,
    size: u64,
) {
    panic!("wgpuRenderBundleEncoderSetIndexBuffer must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderSetLabel(
    renderBundleEncoder: WGPURenderBundleEncoder,
    label: WGPUStringView,
) {
    panic!("wgpuRenderBundleEncoderSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderSetPipeline(
    renderBundleEncoder: WGPURenderBundleEncoder,
    pipeline: WGPURenderPipeline,
) {
    panic!("wgpuRenderBundleEncoderSetPipeline must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderSetVertexBuffer(
    renderBundleEncoder: WGPURenderBundleEncoder,
    slot: u32,
    buffer: WGPUBuffer,
    offset: u64,
    size: u64,
) {
    panic!("wgpuRenderBundleEncoderSetVertexBuffer must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderAddRef(
    renderBundleEncoder: WGPURenderBundleEncoder,
) {
    panic!("wgpuRenderBundleEncoderAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderBundleEncoderRelease(
    renderBundleEncoder: WGPURenderBundleEncoder,
) {
    panic!("wgpuRenderBundleEncoderRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderBeginOcclusionQuery(
    renderPassEncoder: WGPURenderPassEncoder,
    queryIndex: u32,
) {
    panic!("wgpuRenderPassEncoderBeginOcclusionQuery must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderDraw(
    renderPassEncoder: WGPURenderPassEncoder,
    vertexCount: u32,
    instanceCount: u32,
    firstVertex: u32,
    firstInstance: u32,
) {
    panic!("wgpuRenderPassEncoderDraw must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderDrawIndexed(
    renderPassEncoder: WGPURenderPassEncoder,
    indexCount: u32,
    instanceCount: u32,
    firstIndex: u32,
    baseVertex: i32,
    firstInstance: u32,
) {
    panic!("wgpuRenderPassEncoderDrawIndexed must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderDrawIndexedIndirect(
    renderPassEncoder: WGPURenderPassEncoder,
    indirectBuffer: WGPUBuffer,
    indirectOffset: u64,
) {
    panic!("wgpuRenderPassEncoderDrawIndexedIndirect must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderDrawIndirect(
    renderPassEncoder: WGPURenderPassEncoder,
    indirectBuffer: WGPUBuffer,
    indirectOffset: u64,
) {
    panic!("wgpuRenderPassEncoderDrawIndirect must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderEnd(renderPassEncoder: WGPURenderPassEncoder) {
    let pass_ref = unsafe { renderPassEncoder.as_ref().expect("Invalid renderPassEncoder") };
    pass_ref.0.lock().expect("render pass lock poisoned").take(); // dropping the RenderPass ends it
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderEndOcclusionQuery(
    renderPassEncoder: WGPURenderPassEncoder,
) {
    panic!("wgpuRenderPassEncoderEndOcclusionQuery must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderExecuteBundles(
    renderPassEncoder: WGPURenderPassEncoder,
    bundleCount: usize,
    bundles: *const WGPURenderBundle,
) {
    panic!("wgpuRenderPassEncoderExecuteBundles must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderInsertDebugMarker(
    renderPassEncoder: WGPURenderPassEncoder,
    markerLabel: WGPUStringView,
) {
    panic!("wgpuRenderPassEncoderInsertDebugMarker must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderPopDebugGroup(
    renderPassEncoder: WGPURenderPassEncoder,
) {
    panic!("wgpuRenderPassEncoderPopDebugGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderPushDebugGroup(
    renderPassEncoder: WGPURenderPassEncoder,
    groupLabel: WGPUStringView,
) {
    panic!("wgpuRenderPassEncoderPushDebugGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderSetBindGroup(
    renderPassEncoder: WGPURenderPassEncoder,
    groupIndex: u32,
    group: WGPUBindGroup,
    dynamicOffsetCount: usize,
    dynamicOffsets: *const u32,
) {
    panic!("wgpuRenderPassEncoderSetBindGroup must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderSetBlendConstant(
    renderPassEncoder: WGPURenderPassEncoder,
    color: *const WGPUColor,
) {
    panic!("wgpuRenderPassEncoderSetBlendConstant must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderSetIndexBuffer(
    renderPassEncoder: WGPURenderPassEncoder,
    buffer: WGPUBuffer,
    format: WGPUIndexFormat,
    offset: u64,
    size: u64,
) {
    panic!("wgpuRenderPassEncoderSetIndexBuffer must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderSetLabel(
    renderPassEncoder: WGPURenderPassEncoder,
    label: WGPUStringView,
) {
    panic!("wgpuRenderPassEncoderSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderSetPipeline(
    renderPassEncoder: WGPURenderPassEncoder,
    pipeline: WGPURenderPipeline,
) {
    panic!("wgpuRenderPassEncoderSetPipeline must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderSetScissorRect(
    renderPassEncoder: WGPURenderPassEncoder,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) {
    let pass_ref = unsafe { renderPassEncoder.as_ref().expect("Invalid renderPassEncoder") };
    pass_ref
        .0
        .lock()
        .expect("render pass lock poisoned")
        .as_mut()
        .expect("render pass already ended")
        .set_scissor_rect(x, y, width, height);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderSetStencilReference(
    renderPassEncoder: WGPURenderPassEncoder,
    reference: u32,
) {
    panic!("wgpuRenderPassEncoderSetStencilReference must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderSetVertexBuffer(
    renderPassEncoder: WGPURenderPassEncoder,
    slot: u32,
    buffer: WGPUBuffer,
    offset: u64,
    size: u64,
) {
    panic!("wgpuRenderPassEncoderSetVertexBuffer must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderSetViewport(
    renderPassEncoder: WGPURenderPassEncoder,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    minDepth: f32,
    maxDepth: f32,
) {
    let pass_ref = unsafe { renderPassEncoder.as_ref().expect("Invalid renderPassEncoder") };
    pass_ref
        .0
        .lock()
        .expect("render pass lock poisoned")
        .as_mut()
        .expect("render pass already ended")
        .set_viewport(x, y, width, height, minDepth, maxDepth);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderAddRef(renderPassEncoder: WGPURenderPassEncoder) {
    panic!("wgpuRenderPassEncoderAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPassEncoderRelease(renderPassEncoder: WGPURenderPassEncoder) {
    let _ = unsafe { renderPassEncoder.as_ref().expect("Invalid renderPassEncoder") };
    unsafe {
        drop(Arc::from_raw(renderPassEncoder));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPipelineGetBindGroupLayout(
    renderPipeline: WGPURenderPipeline,
    groupIndex: u32,
) -> WGPUBindGroupLayout {
    let pipeline_ref = unsafe { renderPipeline.as_ref().expect("Invalid renderPipeline") };
    let layout = pipeline_ref.0.get_bind_group_layout(groupIndex);
    WGPUBindGroupLayoutImpl(layout).to_pointer()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPipelineSetLabel(
    renderPipeline: WGPURenderPipeline,
    label: WGPUStringView,
) {
    panic!("wgpuRenderPipelineSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPipelineAddRef(renderPipeline: WGPURenderPipeline) {
    panic!("wgpuRenderPipelineAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuRenderPipelineRelease(renderPipeline: WGPURenderPipeline) {
    panic!("wgpuRenderPipelineRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSamplerSetLabel(sampler: WGPUSampler, label: WGPUStringView) {
    panic!("wgpuSamplerSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSamplerAddRef(sampler: WGPUSampler) {
    panic!("wgpuSamplerAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSamplerRelease(sampler: WGPUSampler) {
    panic!("wgpuSamplerRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuShaderModuleGetCompilationInfo(
    shaderModule: WGPUShaderModule,
    callbackInfo: WGPUCompilationInfoCallbackInfo,
) -> WGPUFuture {
    panic!("wgpuShaderModuleGetCompilationInfo must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuShaderModuleSetLabel(
    shaderModule: WGPUShaderModule,
    label: WGPUStringView,
) {
    panic!("wgpuShaderModuleSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuShaderModuleAddRef(shaderModule: WGPUShaderModule) {
    panic!("wgpuShaderModuleAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuShaderModuleRelease(shaderModule: WGPUShaderModule) {
    panic!("wgpuShaderModuleRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSupportedFeaturesFreeMembers(
    supportedFeatures: WGPUSupportedFeatures,
) {
    panic!("wgpuSupportedFeaturesFreeMembers must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSupportedInstanceFeaturesFreeMembers(
    supportedInstanceFeatures: WGPUSupportedInstanceFeatures,
) {
    panic!("wgpuSupportedInstanceFeaturesFreeMembers must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSupportedWGSLLanguageFeaturesFreeMembers(
    supportedWGSLLanguageFeatures: WGPUSupportedWGSLLanguageFeatures,
) {
    panic!("wgpuSupportedWGSLLanguageFeaturesFreeMembers must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSurfaceConfigure(
    surface: WGPUSurface,
    config: *const WGPUSurfaceConfiguration,
) {
    panic!("wgpuSurfaceConfigure must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSurfaceGetCapabilities(
    surface: WGPUSurface,
    adapter: WGPUAdapter,
    capabilities: *mut WGPUSurfaceCapabilities,
) -> WGPUStatus {
    panic!("wgpuSurfaceGetCapabilities must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSurfaceGetCurrentTexture(
    surface: WGPUSurface,
    surfaceTexture: *mut WGPUSurfaceTexture,
) {
    panic!("wgpuSurfaceGetCurrentTexture must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSurfacePresent(surface: WGPUSurface) -> WGPUStatus {
    panic!("wgpuSurfacePresent must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSurfaceSetLabel(surface: WGPUSurface, label: WGPUStringView) {
    panic!("wgpuSurfaceSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSurfaceUnconfigure(surface: WGPUSurface) {
    panic!("wgpuSurfaceUnconfigure must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSurfaceAddRef(surface: WGPUSurface) {
    panic!("wgpuSurfaceAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSurfaceRelease(surface: WGPUSurface) {
    panic!("wgpuSurfaceRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuSurfaceCapabilitiesFreeMembers(
    surfaceCapabilities: WGPUSurfaceCapabilities,
) {
    panic!("wgpuSurfaceCapabilitiesFreeMembers must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureCreateView(
    texture: WGPUTexture,
    descriptor: *const WGPUTextureViewDescriptor,
) -> WGPUTextureView {
    let texture_ref = unsafe { texture.as_ref().expect("Invalid texture") };
    let default_desc = wgpu::TextureViewDescriptor {
        label: None,
        format: None,
        dimension: None,
        usage: None,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    };
    let wgpu_desc = match unsafe { descriptor.as_ref() } {
        Some(d) => conv::texture_view_descriptor(d),
        None => default_desc,
    };
    let view = texture_ref.0.create_view(&wgpu_desc);
    WGPUTextureViewImpl(view).to_pointer()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureDestroy(texture: WGPUTexture) {
    panic!("wgpuTextureDestroy must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureGetDepthOrArrayLayers(texture: WGPUTexture) -> u32 {
    panic!("wgpuTextureGetDepthOrArrayLayers must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureGetDimension(texture: WGPUTexture) -> WGPUTextureDimension {
    panic!("wgpuTextureGetDimension must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureGetFormat(texture: WGPUTexture) -> WGPUTextureFormat {
    panic!("wgpuTextureGetFormat must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureGetHeight(texture: WGPUTexture) -> u32 {
    panic!("wgpuTextureGetHeight must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureGetMipLevelCount(texture: WGPUTexture) -> u32 {
    panic!("wgpuTextureGetMipLevelCount must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureGetSampleCount(texture: WGPUTexture) -> u32 {
    panic!("wgpuTextureGetSampleCount must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureGetTextureBindingViewDimension(
    texture: WGPUTexture,
) -> WGPUTextureViewDimension {
    panic!("wgpuTextureGetTextureBindingViewDimension must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureGetUsage(texture: WGPUTexture) -> WGPUTextureUsage {
    panic!("wgpuTextureGetUsage must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureGetWidth(texture: WGPUTexture) -> u32 {
    panic!("wgpuTextureGetWidth must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureSetLabel(texture: WGPUTexture, label: WGPUStringView) {
    panic!("wgpuTextureSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureAddRef(texture: WGPUTexture) {
    panic!("wgpuTextureAddRef must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureRelease(texture: WGPUTexture) {
    panic!("wgpuTextureRelease must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureViewSetLabel(
    textureView: WGPUTextureView,
    label: WGPUStringView,
) {
    panic!("wgpuTextureViewSetLabel must be implemented");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureViewAddRef(textureView: WGPUTextureView) {
    let _ = unsafe { textureView.as_ref().expect("Invalid textureView") };
    unsafe {
        Arc::increment_strong_count(textureView);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wgpuTextureViewRelease(textureView: WGPUTextureView) {
    let _ = unsafe { textureView.as_ref().expect("Invalid textureView") };
    unsafe {
        drop(Arc::from_raw(textureView));
    }
}
