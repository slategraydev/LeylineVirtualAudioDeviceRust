// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

#![no_std]

use crate::buffer::RingBuffer;
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
    format: *mut KSDATAFORMAT,
}

// ============================================================================
// WaveRT Stream Implementation
// ============================================================================
// Domain logic for stream state control, position reporting, and
// resource management.

impl MiniportWaveRTStream {
    /// # Safety
    /// The caller must ensure the buffer pointer is valid for the given size.
    pub unsafe fn new(buffer_ptr: *mut u8, size: usize, format: *mut KSDATAFORMAT) -> Self {
        Self {
            buffer: RingBuffer::new(buffer_ptr, size),
            state: KSSTATE_STOP,
            format,
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

    /// Allocates kernel memory for the audio buffer.
    pub fn allocate_audio_buffer(
        &mut self,
        _requested_size: usize,
        _audio_buffer_mdl: *mut PMDL,
    ) -> NTSTATUS {
        // Future: Direct Memory Access (DMA) buffer allocation.
        STATUS_SUCCESS
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
        // SAFETY: The buffer was allocated with ExAllocatePool2 in
        // MiniportWaveRT::new_stream using the "LLAD" tag.
        unsafe {
            ExFreePoolWithTag(self.buffer.get_ptr() as PVOID, u32::from_be_bytes(*b"LLAD"));
        }
    }
}
