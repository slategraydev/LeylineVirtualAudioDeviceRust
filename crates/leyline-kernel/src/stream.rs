// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

use crate::buffer::RingBuffer;
use wdk_sys::ntddk::*;
use wdk_sys::*;

// ============================================================================
// Constants
// ============================================================================
// Driver-specific constants and defaults.

/// Default hardware latency in 100-nanosecond units.
const DEFAULT_HW_LATENCY: u32 = 10;

/// KSSTATE enumeration values.
const KSSTATE_STOP: i32 = 0;
const KSSTATE_ACQUIRE: i32 = 1;
const KSSTATE_PAUSE: i32 = 2;
const KSSTATE_RUN: i32 = 3;

// ============================================================================
// WaveRT Stream Structure
// ============================================================================
// Represents a single audio stream instance (pin) on the virtual adapter.

pub struct MiniportWaveRTStream {
    buffer: RingBuffer,
    state: i32,
    _format: PVOID,
    mdl: PMDL,
    mapping: PVOID,
}

// ============================================================================
// WaveRT Stream Implementation
// ============================================================================
// Domain logic for stream state control, position reporting, and
// Resource management for audio streams.

// Resource management for audio streams.

impl MiniportWaveRTStream {
    /// # Safety
    /// The caller must ensure the format pointer is valid.
    pub unsafe fn new(format: PVOID) -> Self {
        Self {
            buffer: RingBuffer::new(core::ptr::null_mut(), 0),
            state: KSSTATE_STOP,
            _format: format,
            mdl: core::ptr::null_mut(),
            mapping: core::ptr::null_mut(),
        }
    }

    /// Sets the current state of the stream.
    pub fn set_state(&mut self, state: i32) -> NTSTATUS {
        self.state = state;

        match state {
            KSSTATE_STOP => {
                // Resetting the buffer ensures subsequent playback
                // starts from a clean state.
                self.buffer.reset();
            }
            KSSTATE_ACQUIRE => {
                // Future: Resource acquisition (e.g., MMCSS registration).
            }
            KSSTATE_PAUSE => {
                // Future: Suspend DMA processing.
            }
            KSSTATE_RUN => {
                // Future: Signal DMA engine to commence.
            }
            _ => return STATUS_INVALID_PARAMETER,
        }
        STATUS_SUCCESS
    }

    /// Reports the current read/write position of the stream.
    pub fn get_position(&self, position: *mut u64) -> NTSTATUS {
        if position.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        // SAFETY: The position pointer is verified non-null.
        unsafe {
            *position = self.buffer.available_read() as u64;
        }
        STATUS_SUCCESS
    }

    /// Allocates kernel memory for the audio buffer using MDLs.
    pub fn allocate_audio_buffer(
        &mut self,
        requested_size: usize,
        audio_buffer_mdl: *mut PMDL,
    ) -> NTSTATUS {
        // Hoist local variables
        let low_address: PHYSICAL_ADDRESS;
        let mut high_address: PHYSICAL_ADDRESS;
        let skip_bytes: PHYSICAL_ADDRESS;
        let total_bytes: u64;

        if audio_buffer_mdl.is_null() {
            return STATUS_INVALID_PARAMETER;
        }

        total_bytes = requested_size as u64;

        // SAFETY: Initialize physical address range for allocation.
        unsafe {
            low_address = core::mem::zeroed();
            high_address = core::mem::zeroed();
            high_address.QuadPart = 0xFFFFFFFF; // 4GB range
            skip_bytes = core::mem::zeroed();
        }

        // SAFETY: MmAllocatePagesForMdlEx allocates physical pages for the buffer.
        unsafe {
            self.mdl = MmAllocatePagesForMdlEx(
                low_address,
                high_address,
                skip_bytes,
                total_bytes,
                _MEMORY_CACHING_TYPE::MmCached,
                MM_ALLOCATE_FULLY_REQUIRED,
            );
        }

        if self.mdl.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        // SAFETY: Map the MDL to a system virtual address for driver access.
        unsafe {
            self.mapping = MmMapLockedPagesSpecifyCache(
                self.mdl,
                0, // KernelMode
                _MEMORY_CACHING_TYPE::MmCached,
                core::ptr::null_mut(),
                0,
                _MM_PAGE_PRIORITY::NormalPagePriority as u32,
            );
        }

        if self.mapping.is_null() {
            unsafe {
                MmFreePagesFromMdl(self.mdl);
                IoFreeMdl(self.mdl);
                self.mdl = core::ptr::null_mut();
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        // Rebase the ring buffer to the new mapping
        unsafe {
            self.buffer.rebase(self.mapping as *mut u8, requested_size);
            *audio_buffer_mdl = self.mdl;
        }

        STATUS_SUCCESS
    }

    /// Maps the audio buffer to the specified process's user address space.
    pub fn map_user_buffer(&mut self, _process: PEPROCESS) -> *mut u8 {
        if self.mdl.is_null() {
            return core::ptr::null_mut();
        }

        // SAFETY: Map the MDL to user-space for zero-copy access from the APO.
        unsafe {
            MmMapLockedPagesSpecifyCache(
                self.mdl,
                1, // UserMode
                _MEMORY_CACHING_TYPE::MmCached,
                core::ptr::null_mut(),
                0,
                _MM_PAGE_PRIORITY::NormalPagePriority as u32,
            ) as *mut u8
        }
    }

    /// Returns the hardware latency estimate for the stream.
    pub fn get_hw_latency(&self, latency: *mut u32) {
        if !latency.is_null() {
            // SAFETY: The latency pointer is verified non-null.
            unsafe {
                *latency = DEFAULT_HW_LATENCY;
            }
        }
    }
}

// ============================================================================
// Resource Cleanup (RAII)
// ============================================================================
impl Drop for MiniportWaveRTStream {
    fn drop(&mut self) {
        if !self.mdl.is_null() {
            // SAFETY: Clean up MDL and mapping.
            unsafe {
                if !self.mapping.is_null() {
                    MmUnmapLockedPages(self.mapping, self.mdl);
                }
                MmFreePagesFromMdl(self.mdl);
                IoFreeMdl(self.mdl);
            }
        }
    }
}
