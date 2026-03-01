// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// WAVERT STREAMING & BUFFER MANAGEMENT
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#![allow(non_camel_case_types)]

use alloc::boxed::Box;
use core::mem::zeroed;
use core::ptr::null_mut;

use wdk_sys::ntddk::*;
use wdk_sys::*;

use crate::adapter::DeviceExtension;

use crate::audio_bindings as audio;

// Re-export specific audio types.
#[allow(non_camel_case_types)]
pub type PCFILTER_DESCRIPTOR = audio::PCFILTER_DESCRIPTOR;
#[allow(non_camel_case_types)]
pub type PCPIN_DESCRIPTOR = audio::PCPIN_DESCRIPTOR;
#[allow(non_camel_case_types)]
pub type PCAUTOMATION_TABLE = audio::PCAUTOMATION_TABLE;
#[allow(non_camel_case_types)]
pub type WAVEFORMATEX = audio::WAVEFORMATEX;
#[allow(non_camel_case_types)]
pub type KSPIN_DESCRIPTOR = audio::KSPIN_DESCRIPTOR;
#[allow(non_camel_case_types)]
pub type KSDATAFORMAT = audio::KSDATAFORMAT;
#[allow(non_camel_case_types)]
pub type KSDATARANGE = audio::KSDATARANGE;
#[allow(non_camel_case_types)]
pub type PKSDATARANGE = audio::PKSDATARANGE;
#[allow(non_camel_case_types)]
pub type PCCONNECTION = audio::PCCONNECTION_DESCRIPTOR;
#[allow(non_camel_case_types)]
pub type PCNODE_DESCRIPTOR = audio::PCNODE_DESCRIPTOR;
#[allow(non_camel_case_types)]
pub type PCPROPERTY_ITEM = audio::PCPROPERTY_ITEM;
#[allow(non_camel_case_types)]
pub type PPCPROPERTY_REQUEST = audio::PPCPROPERTY_REQUEST;

pub const KSSTATE_RUN: i32 = audio::KSSTATE_KSSTATE_RUN;
pub const KSSTATE_STOP: i32 = audio::KSSTATE_KSSTATE_STOP;

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// WAVERT STRUCT DEFINITIONS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSDATAFORMAT_WAVEFORMATEX {
    pub DataFormat: KSDATAFORMAT,
    pub WaveFormatEx: WAVEFORMATEX,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSDATAFORMAT_WAVEFORMATEXTENSIBLE {
    pub DataFormat: KSDATAFORMAT,
    pub WaveFormatExt: audio::WAVEFORMATEXTENSIBLE,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSDATARANGE_AUDIO {
    pub DataRange: KSDATARANGE,
    pub MaximumChannels: ULONG,
    pub MinimumBitsPerSample: ULONG,
    pub MaximumBitsPerSample: ULONG,
    pub MinimumSampleFrequency: ULONG,
    pub MaximumSampleFrequency: ULONG,
}

unsafe impl Sync for KSDATARANGE_AUDIO {}
unsafe impl Sync for PCPIN_DESCRIPTOR {}
unsafe impl Sync for PCNODE_DESCRIPTOR {}
unsafe impl Sync for PCFILTER_DESCRIPTOR {}
unsafe impl Sync for PCAUTOMATION_TABLE {}
unsafe impl Sync for PCPROPERTY_ITEM {}

pub trait TimeSource {
    fn query_time(&self) -> i64;
    fn query_frequency(&self) -> i64;
}

pub struct KernelTimeSource;

impl TimeSource for KernelTimeSource {
    fn query_time(&self) -> i64 {
        unsafe {
            let counter = KeQueryPerformanceCounter(null_mut());
            counter.QuadPart
        }
    }
    fn query_frequency(&self) -> i64 {
        let mut frequency: LARGE_INTEGER = unsafe { zeroed() };
        unsafe {
            KeQueryPerformanceCounter(&mut frequency);
            frequency.QuadPart
        }
    }
}

pub struct MiniportWaveRTStream {
    buffer: leyline_shared::buffer::RingBuffer,
    state: i32,
    _format: PVOID,
    mdl: PMDL,
    mapping: PVOID,
    _is_capture: bool,
    start_time: i64,
    byte_rate: u32,
    frequency: i64,
    time_source: Box<dyn TimeSource>,
    pub device_extension: *mut u8,
    owns_mdl: bool,
}

impl MiniportWaveRTStream {
    /// Create a new MiniportWaveRTStream instance.
    ///
    /// # Safety
    /// The provided format pointer must be valid for the duration of the stream's lifetime.
    /// The device_extension pointer must be a valid pointer to a DeviceExtension struct.
    pub unsafe fn new(
        format: PVOID,
        is_capture: bool,
        time_source: Box<dyn TimeSource>,
        device_extension: *mut u8,
    ) -> Self {
        let mut byte_rate: u32 = 48000 * 4;
        let frequency = time_source.query_frequency();
        if !format.is_null() {
            let wave_format = format as *const KSDATAFORMAT_WAVEFORMATEX;
            byte_rate = (*wave_format).WaveFormatEx.nAvgBytesPerSec;
        }
        Self {
            buffer: leyline_shared::buffer::RingBuffer::new(null_mut(), 0),
            state: KSSTATE_STOP,
            _format: format,
            mdl: null_mut(),
            mapping: null_mut(),
            _is_capture: is_capture,
            start_time: 0,
            byte_rate,
            frequency,
            time_source,
            device_extension,
            owns_mdl: false,
        }
    }

    pub fn set_state(&mut self, state: i32) -> NTSTATUS {
        self.state = state;
        if state == KSSTATE_STOP {
            self.start_time = 0;
        } else if state == KSSTATE_RUN {
            self.start_time = self.time_source.query_time();
        }
        STATUS_SUCCESS
    }

    /// Retrieve the current play/record position.
    ///
    /// # Safety
    /// The provided position pointer must be a valid pointer to a u64.
    pub unsafe fn get_position(&mut self, position: *mut u64) -> NTSTATUS {
        if self.state != KSSTATE_RUN || self.start_time == 0 {
            if !position.is_null() {
                *position = 0;
            }
            return STATUS_SUCCESS;
        }

        let now = self.time_source.query_time();
        let elapsed_ticks = now - self.start_time;
        let elapsed_bytes = leyline_shared::math::WaveRTMath::ticks_to_bytes(
            elapsed_ticks,
            self.byte_rate,
            self.frequency,
        );

        if !position.is_null() {
            if !self.buffer.get_base_address().is_null() {
                *position = elapsed_bytes % (self.buffer.get_size() as u64);
            } else {
                *position = 0;
            }
        }
        STATUS_SUCCESS
    }

    /// Allocate the audio buffer for WaveRT streaming.
    ///
    /// # Safety
    /// The provided out_mdl must be a valid pointer to a PMDL.
    pub unsafe fn allocate_audio_buffer(&mut self, size: usize, out_mdl: *mut PMDL) -> NTSTATUS {
        if !self.mdl.is_null() {
            return STATUS_ALREADY_COMMITTED;
        }

        let low: PHYSICAL_ADDRESS = zeroed();
        let mut high: PHYSICAL_ADDRESS = zeroed();
        high.QuadPart = 0xFFFFFFFF;
        let skip: PHYSICAL_ADDRESS = zeroed();

        let mdl = MmAllocatePagesForMdlEx(
            low,
            high,
            skip,
            size as u64,
            _MEMORY_CACHING_TYPE::MmCached,
            MM_ALLOCATE_FULLY_REQUIRED,
        );

        if mdl.is_null() {
            // Fallback to device extension loopback if available.
            if !self.device_extension.is_null() {
                let device_extension = self.device_extension as *mut DeviceExtension;
                if !(*device_extension).loopback_mdl.is_null() {
                    self.mdl = (*device_extension).loopback_mdl;
                    self.mapping = (*device_extension).loopback_buffer as PVOID;
                    self.buffer = leyline_shared::buffer::RingBuffer::new(
                        self.mapping as *mut u8,
                        (*device_extension).loopback_size,
                    );
                    if !out_mdl.is_null() {
                        *out_mdl = self.mdl;
                    }
                    self.owns_mdl = false;
                    return STATUS_SUCCESS;
                }
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        self.mapping = MmMapLockedPagesSpecifyCache(
            mdl,
            0, // KernelMode.
            _MEMORY_CACHING_TYPE::MmCached,
            null_mut(),
            0,
            _MM_PAGE_PRIORITY::NormalPagePriority as u32,
        ) as PVOID;

        if self.mapping.is_null() {
            IoFreeMdl(mdl);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        self.mdl = mdl;
        self.buffer = leyline_shared::buffer::RingBuffer::new(self.mapping as *mut u8, size);
        self.owns_mdl = true;
        if !out_mdl.is_null() {
            *out_mdl = mdl;
        }

        STATUS_SUCCESS
    }

    /// Retrieve the hardware-specific latency.
    ///
    /// # Safety
    /// The provided latency pointer must be a valid pointer to a u32.
    pub unsafe fn get_hw_latency(&self, latency: *mut u32) {
        if !latency.is_null() {
            *latency = 0; // Software-only driver.
        }
    }
}

impl Drop for MiniportWaveRTStream {
    fn drop(&mut self) {
        unsafe {
            if self.owns_mdl && !self.mdl.is_null() {
                if !self.mapping.is_null() {
                    MmUnmapLockedPages(self.mapping, self.mdl);
                }
                MmFreePagesFromMdl(self.mdl);
                IoFreeMdl(self.mdl);
            }
        }
    }
}

