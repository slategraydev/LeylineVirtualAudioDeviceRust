// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

#![no_std]

use crate::adapter::DeviceExtension;
use wdk_sys::ntddk::*;
use wdk_sys::*;

// Correct bindings for wdk-sys
pub type KSDATAFORMAT = wdk_sys::_KSDATAFORMAT;
pub type WAVEFORMATEX = wdk_sys::tWAVEFORMATEX;
pub type KSDATARANGE = wdk_sys::_KSDATARANGE;
pub type KSPIN_DESCRIPTOR = wdk_sys::_KSPIN_DESCRIPTOR;

// ============================================================================
// WaveRT Struct Definitions
// ============================================================================

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSDATAFORMAT_WAVEFORMATEX {
    pub DataFormat: KSDATAFORMAT,
    pub WaveFormatEx: WAVEFORMATEX,
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

#[repr(C)]
#[allow(non_snake_case)]
pub struct PCCONNECTION {
    pub FromNode: ULONG,
    pub FromPin: ULONG,
    pub ToNode: ULONG,
    pub ToPin: ULONG,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct PCPIN_DESCRIPTOR {
    pub MaxGlobalInstanceCount: ULONG,
    pub MaxFilterInstanceCount: ULONG,
    pub MinFilterInstanceCount: ULONG,
    pub AutomationTable: *const u8,
    pub KsPinDescriptor: KSPIN_DESCRIPTOR,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct PCFILTER_DESCRIPTOR {
    pub Version: ULONG,
    pub AutomationTable: *const u8,
    pub PinSize: ULONG,
    pub PinCount: ULONG,
    pub Pins: *const PCPIN_DESCRIPTOR,
    pub NodeSize: ULONG,
    pub NodeCount: ULONG,
    pub Nodes: *const u8,
    pub ConnectionCount: ULONG,
    pub Connections: *const PCCONNECTION,
    pub CategoryCount: ULONG,
    pub Categories: *const GUID,
}

unsafe impl Sync for KSDATARANGE_AUDIO {}
unsafe impl Sync for PCPIN_DESCRIPTOR {}
unsafe impl Sync for PCFILTER_DESCRIPTOR {}

pub trait TimeSource {
    fn query_time(&self) -> i64;
    fn query_frequency(&self) -> i64;
}

pub struct KernelTimeSource;

impl TimeSource for KernelTimeSource {
    fn query_time(&self) -> i64 {
        unsafe {
            let counter = KeQueryPerformanceCounter(core::ptr::null_mut());
            counter.QuadPart
        }
    }
    fn query_frequency(&self) -> i64 {
        let mut freq: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        unsafe {
            KeQueryPerformanceCounter(&mut freq);
            freq.QuadPart
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
    time_source: alloc::boxed::Box<dyn TimeSource>,
    pub device_extension: *mut u8,
    owns_mdl: bool,
}

impl MiniportWaveRTStream {
    pub unsafe fn new(
        format: PVOID,
        is_capture: bool,
        time_source: alloc::boxed::Box<dyn TimeSource>,
        device_extension: *mut u8,
    ) -> Self {
        let mut byte_rate: u32 = 48000 * 4;
        let frequency = time_source.query_frequency();
        if !format.is_null() {
            let wave_format = format as *const KSDATAFORMAT_WAVEFORMATEX;
            byte_rate = (*wave_format).WaveFormatEx.nAvgBytesPerSec;
        }
        Self {
            buffer: leyline_shared::buffer::RingBuffer::new(core::ptr::null_mut(), 0),
            state: _KSSTATE::KSSTATE_STOP as i32,
            _format: format,
            mdl: core::ptr::null_mut(),
            mapping: core::ptr::null_mut(),
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
        if state == _KSSTATE::KSSTATE_STOP as i32 {
            self.start_time = 0;
        } else if state == _KSSTATE::KSSTATE_RUN as i32 {
            self.start_time = self.time_source.query_time();
        }
        STATUS_SUCCESS
    }

    pub fn get_position(&mut self, position: *mut u64) -> NTSTATUS {
        if self.state != _KSSTATE::KSSTATE_RUN as i32 || self.start_time == 0 {
            unsafe {
                if !position.is_null() {
                    *position = 0;
                }
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

        unsafe {
            if !position.is_null() {
                if !self.buffer.get_base_address().is_null() {
                    *position = elapsed_bytes % (self.buffer.get_size() as u64);
                } else {
                    *position = 0;
                }
            }
        }
        STATUS_SUCCESS
    }

    pub unsafe fn allocate_audio_buffer(&mut self, size: usize, out_mdl: *mut PMDL) -> NTSTATUS {
        if !self.mdl.is_null() {
            return STATUS_ALREADY_COMMITTED;
        }

        let low: PHYSICAL_ADDRESS = core::mem::zeroed();
        let mut high: PHYSICAL_ADDRESS = core::mem::zeroed();
        high.QuadPart = 0xFFFFFFFF;
        let skip: PHYSICAL_ADDRESS = core::mem::zeroed();

        let mdl = MmAllocatePagesForMdlEx(
            low,
            high,
            skip,
            size as u64,
            _MEMORY_CACHING_TYPE::MmCached,
            MM_ALLOCATE_FULLY_REQUIRED,
        );

        if mdl.is_null() {
            // Fallback to device extension loopback if available
            if !self.device_extension.is_null() {
                let dev_ext = self.device_extension as *mut DeviceExtension;
                if !(*dev_ext).loopback_mdl.is_null() {
                    self.mdl = (*dev_ext).loopback_mdl;
                    self.mapping = (*dev_ext).loopback_buffer as PVOID;
                    self.buffer = leyline_shared::buffer::RingBuffer::new(
                        self.mapping as *mut u8,
                        (*dev_ext).loopback_size,
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
            0, // KernelMode
            _MEMORY_CACHING_TYPE::MmCached,
            core::ptr::null_mut(),
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

    pub fn get_hw_latency(&self, latency: *mut u32) {
        unsafe {
            if !latency.is_null() {
                *latency = 0; // Software-only driver
            }
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
                IoFreeMdl(self.mdl);
            }
        }
    }
}
