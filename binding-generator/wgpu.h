/*
 * Copyright 2024 wgpu-native contributors
 * Copyright 2026 MapLibre contributors
 *
 * This file contains code copied from wgpu-native (https://github.com/gfx-rs/wgpu-native)
 * Licensed under the Apache License, Version 2.0 or the MIT License, at your option.
 */

#ifndef WGPU_H_
#define WGPU_H_

#include "webgpu.h"

/**
 * Identifier for a particular call to @ref wgpuQueueSubmitForIndex.
 *
 * Can be passed to @ref wgpuDevicePoll to block until a particular
 * submission has finished execution.
 *
 * This type is unique to wgpu-native; there is no analogue in the
 * WebGPU specification.
 */
typedef uint64_t WGPUSubmissionIndex;

WGPUBool wgpuDevicePoll(WGPUDevice device, WGPUBool wait, WGPU_NULLABLE WGPUSubmissionIndex const *submissionIndex);

#endif // WGPU_H_