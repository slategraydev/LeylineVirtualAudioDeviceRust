// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// PORTCLS INTERFACE VTABLE DEFINITIONS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use wdk_sys::ntddk::*;
use wdk_sys::*;

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

#[allow(non_snake_case)]
#[repr(C)]
pub struct IPinCountVTable {
    pub base: IUnknownVTable,
    pub PinCount: unsafe extern "system" fn(
        this: *mut u8,
        pin_id: u32,
        filter_necessary: *mut u32,
        filter_current: *mut u32,
        filter_possible: *mut u32,
        global_current: *mut u32,
        global_possible: *mut u32,
    ),
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IPinNameVTable {
    pub base: IUnknownVTable,
    pub GetPinName: unsafe extern "system" fn(
        this: *mut u8,
        irp: *mut u8,
        pin: *mut u8,
        data: *mut u8,
    ) -> NTSTATUS,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IMiniportWaveRTOutputStreamVTable {
    pub base: IUnknownVTable,
    pub SetWritePacket: unsafe extern "system" fn(
        this: *mut u8,
        packet_number: u32,
        flags: u32,
        eos_packet_length: u32,
    ) -> NTSTATUS,
    pub GetOutputStreamPresentationPosition: unsafe extern "system" fn(
        this: *mut u8,
        presentation_position: *mut u64,
        performance_counter: *mut u64,
    ) -> NTSTATUS,
    pub GetPacketCount:
        unsafe extern "system" fn(this: *mut u8, packet_count: *mut u32) -> NTSTATUS,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IMiniportWaveRTInputStreamVTable {
    pub base: IUnknownVTable,
    pub GetReadPacket: unsafe extern "system" fn(
        this: *mut u8,
        packet_number: *mut u32,
        flags: *mut u32,
        performance_counter: *mut u64,
        more_data: *mut i32,
    ) -> NTSTATUS,
}
#[allow(non_snake_case)]
#[repr(C)]
pub struct IPortClsStreamResourceManager2VTable {
    pub base: IUnknownVTable,
    pub AddResource: unsafe extern "system" fn(
        this: *mut u8,
        resource_description: *mut u8,
        resource_handle: *mut *mut u8,
    ) -> NTSTATUS,
    pub RemoveResource:
        unsafe extern "system" fn(this: *mut u8, resource_handle: *mut u8) -> NTSTATUS,
}

