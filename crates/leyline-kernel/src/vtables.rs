// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

// Second, external crates.
use wdk_sys::{GUID, NTSTATUS};

#[allow(non_snake_case)]
#[repr(C)]
pub struct IUnknownVTable {
    pub QueryInterface:
        unsafe extern "system" fn(this: *mut u8, iid: *const GUID, out: *mut *mut u8) -> NTSTATUS,
    pub AddRef: unsafe extern "system" fn(this: *mut u8) -> u32,
    pub Release: unsafe extern "system" fn(this: *mut u8) -> u32,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IMiniportTopologyVTable {
    pub base: IUnknownVTable,
    pub GetDescription:
        unsafe extern "system" fn(this: *mut u8, out_description: *mut u8) -> NTSTATUS,
    pub DataRangeIntersection: unsafe extern "system" fn(
        this: *mut u8,
        pin_id: u32,
        data_range: *mut u8,
        matching_data_range: *mut u8,
        data_format_cb: u32,
        data_format: *mut u8,
        actual_data_format_cb: *mut u32,
    ) -> NTSTATUS,
    pub Init: unsafe extern "system" fn(
        this: *mut u8,
        unknown_adapter: *mut u8,
        resource_list: *mut u8,
        port: *mut u8,
    ) -> NTSTATUS,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IMiniportWaveRTVTable {
    pub base: IUnknownVTable,
    pub GetDescription:
        unsafe extern "system" fn(this: *mut u8, out_description: *mut u8) -> NTSTATUS,
    pub DataRangeIntersection: unsafe extern "system" fn(
        this: *mut u8,
        pin_id: u32,
        data_range: *mut u8,
        matching_data_range: *mut u8,
        data_format_cb: u32,
        data_format: *mut u8,
        actual_data_format_cb: *mut u32,
    ) -> NTSTATUS,
    pub Init: unsafe extern "system" fn(
        this: *mut u8,
        unknown_adapter: *mut u8,
        resource_list: *mut u8,
        port: *mut u8,
    ) -> NTSTATUS,
    pub NewStream: unsafe extern "system" fn(
        this: *mut u8,
        stream: *mut *mut u8,
        port_stream: *mut u8,
        pin: u32,
        capture: bool,
        format: *mut u8,
    ) -> NTSTATUS,
    pub GetDeviceDescription:
        unsafe extern "system" fn(this: *mut u8, description: *mut u8) -> NTSTATUS,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IMiniportWaveRTStreamVTable {
    pub base: IUnknownVTable,
    pub SetFormat: unsafe extern "system" fn(this: *mut u8, format: *mut u8) -> NTSTATUS,
    pub SetState: unsafe extern "system" fn(this: *mut u8, state: i32) -> NTSTATUS,
    pub GetPosition: unsafe extern "system" fn(this: *mut u8, position: *mut u64) -> NTSTATUS,
    pub AllocateAudioBuffer: unsafe extern "system" fn(
        this: *mut u8,
        requested_size: usize,
        audio_buffer_mdl: *mut *mut u8,
        actual_size: *mut usize,
        offset_from_start: *mut u32,
        cache_type: *mut i32,
    ) -> NTSTATUS,
    pub FreeAudioBuffer:
        unsafe extern "system" fn(this: *mut u8, audio_buffer_mdl: *mut u8, buffer_size: usize),
    pub GetHWLatency: unsafe extern "system" fn(this: *mut u8, latency: *mut u32),
    pub GetPositionRegister:
        unsafe extern "system" fn(this: *mut u8, position_register: *mut u8) -> NTSTATUS,
    pub GetClockRegister:
        unsafe extern "system" fn(this: *mut u8, clock_register: *mut u8) -> NTSTATUS,
}
