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

const DEFAULT_HW_LATENCY: u32 = 20_000;
const KSSTATE_STOP: i32 = 0;
const KSSTATE_RUN: i32 = 3;

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

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSDATARANGE {
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
pub struct PCPIN_DESCRIPTOR {
    pub MaxGlobalInstanceCount: ULONG,
    pub MaxFilterInstanceCount: ULONG,
    pub MinFilterInstanceCount: ULONG,
    pub AutomationTable: *const u8,
    pub KsPinDescriptor: KSPIN_DESCRIPTOR,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct KSPIN_DESCRIPTOR {
    pub InterfacesCount: ULONG,
    pub Interfaces: *const GUID,
    pub MediumsCount: ULONG,
    pub Mediums: *const u8,
    pub DataRangesCount: ULONG,
    pub DataRanges: *const *const KSDATARANGE,
    pub DataFlow: ULONG,
    pub Communication: ULONG,
    pub Category: *const GUID,
    pub Name: *const GUID,
    pub Reserved: ULONG,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct PCFILTER_DESCRIPTOR {
    pub Version: ULONG,
    pub AutomationTable: *const u8,
    pub PinSize: ULONG,
    pub PinDescriptorSize: ULONG,
    pub Pins: *const PCPIN_DESCRIPTOR,
    pub NodeSize: ULONG,
    pub NodeDescriptorSize: ULONG,
    pub Nodes: *const u8,
    pub ConnectionCount: ULONG,
    pub Connections: *const u8,
    pub CategoryCount: ULONG,
    pub Categories: *const GUID,
}

unsafe impl Sync for KSDATARANGE {}
unsafe impl Sync for KSDATARANGE_AUDIO {}
unsafe impl Sync for PCPIN_DESCRIPTOR {}
unsafe impl Sync for KSPIN_DESCRIPTOR {}
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
    buffer: RingBuffer,
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
            device_extension,
        }
    }

    pub fn set_state(&mut self, state: i32) -> NTSTATUS {
        self.state = state;
        match state {
            KSSTATE_STOP => {
                self.buffer.reset();
                self.start_time = 0;
            }
            KSSTATE_RUN => {
                self.start_time = self.time_source.query_time();
                if !self.device_extension.is_null() {
                    let dev_ext = self.device_extension as *mut crate::DeviceExtension;
                    unsafe {
                        if !(*dev_ext).shared_params.is_null() {
                            let params = (*dev_ext).shared_params;
                            (*params).qpc_frequency = self.frequency;
                            (*params).buffer_size = self.buffer.get_size() as u32;
                            (*params).byte_rate = self.byte_rate;
                            if self._is_capture {
                                (*params).capture_start_qpc = self.start_time;
                            } else {
                                (*params).render_start_qpc = self.start_time;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        STATUS_SUCCESS
    }

    pub fn get_position(&self, position: *mut u64) -> NTSTATUS {
        if position.is_null() {
            return STATUS_INVALID_PARAMETER;
        }
        let mut calculated_position: u64 = 0;
        if self.state == KSSTATE_RUN {
            let current_time = self.time_source.query_time();
            let elapsed_ticks = current_time - self.start_time;
            calculated_position = WaveRTMath::calculate_position(
                elapsed_ticks,
                self.byte_rate,
                self.frequency,
                self.buffer.get_size(),
            );
        }
        unsafe {
            *position = calculated_position;
        }
        STATUS_SUCCESS
    }

    pub fn allocate_audio_buffer(&mut self, req_size: usize, audio_mdl: *mut PMDL) -> NTSTATUS {
        if audio_mdl.is_null() {
            return STATUS_INVALID_PARAMETER;
        }
        if !self.device_extension.is_null() {
            let dev_ext = self.device_extension as *mut crate::DeviceExtension;
            unsafe {
                if !(*dev_ext).loopback_mdl.is_null() {
                    self.mdl = (*dev_ext).loopback_mdl;
                    self.mapping = (*dev_ext).loopback_buffer as PVOID;
                    self.buffer.rebase(self.mapping as *mut u8, req_size);
                    *audio_mdl = self.mdl;
                    return STATUS_SUCCESS;
                }
            }
        }

        let low: PHYSICAL_ADDRESS = unsafe { core::mem::zeroed() };
        let mut high: PHYSICAL_ADDRESS = unsafe { core::mem::zeroed() };
        high.QuadPart = 0xFFFFFFFF;
        let skip: PHYSICAL_ADDRESS = unsafe { core::mem::zeroed() };

        unsafe {
            self.mdl = MmAllocatePagesForMdlEx(
                low,
                high,
                skip,
                req_size as u64,
                _MEMORY_CACHING_TYPE::MmCached,
                MM_ALLOCATE_FULLY_REQUIRED,
            );
            if self.mdl.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            self.mapping = MmMapLockedPagesSpecifyCache(
                self.mdl,
                0,
                _MEMORY_CACHING_TYPE::MmCached,
                core::ptr::null_mut(),
                0,
                _MM_PAGE_PRIORITY::NormalPagePriority as u32,
            );
            if self.mapping.is_null() {
                MmFreePagesFromMdl(self.mdl);
                IoFreeMdl(self.mdl);
                self.mdl = core::ptr::null_mut();
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            self.buffer.rebase(self.mapping as *mut u8, req_size);
            *audio_mdl = self.mdl;
        }
        STATUS_SUCCESS
    }

    pub fn get_hw_latency(&self, latency: *mut u32) {
        if !latency.is_null() {
            unsafe {
                *latency = DEFAULT_HW_LATENCY;
            }
        }
    }
}

impl Drop for MiniportWaveRTStream {
    fn drop(&mut self) {
        if !self.mdl.is_null() {
            let mut is_shared = false;
            if !self.device_extension.is_null() {
                let dev_ext = self.device_extension as *mut crate::DeviceExtension;
                unsafe {
                    if self.mdl == (*dev_ext).loopback_mdl {
                        is_shared = true;
                    }
                }
            }
            if !is_shared {
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
}
