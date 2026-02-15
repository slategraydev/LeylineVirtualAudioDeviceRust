// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

use crate::buffer::RingBuffer;
use crate::math::WaveRTMath;
use wdk_sys::ntddk::*;
use wdk_sys::*;

// ============================================================================
// Constants
// ============================================================================
// Driver-specific constants and defaults.

/// Default hardware latency in 100-nanosecond units.
const DEFAULT_HW_LATENCY: u32 = 20_000; // 2ms

/// KSSTATE enumeration values.
const KSSTATE_STOP: i32 = 0;
const KSSTATE_ACQUIRE: i32 = 1;
const KSSTATE_PAUSE: i32 = 2;
const KSSTATE_RUN: i32 = 3;

// ============================================================================
// KS Data Formats (Manual Definitions)
// ============================================================================
// wdk-sys does not export these by default, so we define them to match the C layout.

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSDATAFORMAT {
    pub FormatSize: ULONG,
    pub Flags: ULONG,
    pub SampleSize: ULONG,
    pub Reserved: ULONG,
    pub MajorFormat: GUID,
    pub SubFormat: GUID,
    pub Specifier: GUID,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct WAVEFORMATEX {
    pub wFormatTag: u16,
    pub nChannels: u16,
    pub nSamplesPerSec: u32,
    pub nAvgBytesPerSec: u32,
    pub nBlockAlign: u16,
    pub wBitsPerSample: u16,
    pub cbSize: u16,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSDATAFORMAT_WAVEFORMATEX {
    pub DataFormat: KSDATAFORMAT,
    pub WaveFormatEx: WAVEFORMATEX,
}

// ============================================================================
// Time Source Abstraction (For Testing)
// ============================================================================

pub trait TimeSource {
    fn query_time(&self) -> i64;
    fn query_frequency(&self) -> i64;
}

pub struct KernelTimeSource;

#[cfg(not(test))]
impl TimeSource for KernelTimeSource {
    fn query_time(&self) -> i64 {
        unsafe {
            let counter = KeQueryPerformanceCounter(core::ptr::null_mut());
            counter.QuadPart
        }
    }

    fn query_frequency(&self) -> i64 {
        let mut frequency: LARGE_INTEGER;
        unsafe {
            frequency = core::mem::zeroed();
            KeQueryPerformanceCounter(&mut frequency);
            frequency.QuadPart
        }
    }
}

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
    // Advanced WaveRT Fields
    _is_capture: bool,
    start_time: i64,
    byte_rate: u32,
    frequency: i64,
    time_source: alloc::boxed::Box<dyn TimeSource>,
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
    pub unsafe fn new(
        format: PVOID,
        is_capture: bool,
        time_source: alloc::boxed::Box<dyn TimeSource>,
    ) -> Self {
        let mut byte_rate: u32 = 192000 * 4; // Default fallback

        let frequency = time_source.query_frequency();

        // SAFETY: Parse the data format to extract the byte rate.
        // We assume KSDATAFORMAT_WAVEFORMATEX for this prototype.
        if !format.is_null() {
            let wave_format = format as *const KSDATAFORMAT_WAVEFORMATEX;
            byte_rate = (*wave_format).WaveFormatEx.nAvgBytesPerSec;
        }

        Self {
            buffer: RingBuffer::new(core::ptr::null_mut(), 0),
            state: KSSTATE_STOP,
            _format: format,
            mdl: core::ptr::null_mut(),
            mapping: core::ptr::null_mut(),
            _is_capture: is_capture,
            start_time: 0,
            byte_rate,
            frequency,
            time_source,
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
                self.start_time = 0;
            }
            KSSTATE_ACQUIRE => {
                // Future: Resource acquisition (e.g., MMCSS registration).
            }
            KSSTATE_PAUSE => {
                // Future: Suspend DMA processing.
            }
            KSSTATE_RUN => {
                // Determine the start time for position calculation.
                self.start_time = self.time_source.query_time();
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

        // Calculate simulated position based on elapsed time.
        // Formula: Bytes = (ElapsedTicks * ByteRate) / Frequency
        let current_time: i64;
        let elapsed_ticks: i64;
        let mut calculated_position: u64;

        if self.state == KSSTATE_RUN {
            current_time = self.time_source.query_time();
            elapsed_ticks = current_time - self.start_time;

            calculated_position = WaveRTMath::calculate_position(
                elapsed_ticks,
                self.byte_rate,
                self.frequency,
                self.buffer.get_size(),
            );
        } else {
            calculated_position = 0;
            if self.state == KSSTATE_PAUSE {
                // In pause, we should ideally hold the last position.
                // For now, we return 0 or the last calculated (todo: store last pos).
                // Returning self.buffer.read_pos for now as a fallback if available.
                calculated_position = self.buffer.available_read() as u64;
            }
        }

        // SAFETY: Return the calculated position
        unsafe {
            *position = calculated_position;
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
