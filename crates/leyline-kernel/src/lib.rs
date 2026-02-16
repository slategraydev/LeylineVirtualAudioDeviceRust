// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

#![no_std]

extern crate alloc;

pub mod adapter;
pub mod constants;
pub mod descriptors;
pub mod dispatch;
pub mod stream;
pub mod topology;
pub mod vtables;
pub mod wavert;

pub use stream::audio;

use crate::adapter::{AddDevice, MiniportWaveRTStreamCom};
use crate::constants::*;
use crate::dispatch::*;
use wdk_alloc::WDKAllocator;
use wdk_sys::ntddk::*;
use wdk_sys::*;

#[global_allocator]
static GLOBAL: WDKAllocator = WDKAllocator;

// ============================================================================
// PortCls External Declarations
// ============================================================================

#[link(name = "portcls")]
extern "C" {
    pub fn PcInitializeAdapterDriver(
        DriverObject: PDRIVER_OBJECT,
        RegistryPath: PUNICODE_STRING,
        AddDevice: Option<unsafe extern "C" fn(PDRIVER_OBJECT, PDEVICE_OBJECT) -> NTSTATUS>,
    ) -> NTSTATUS;

    pub fn PcNewPort(OutPort: *mut *mut u8, ClassId: *const GUID) -> NTSTATUS;

    pub fn PcRegisterSubdevice(
        DeviceObject: PDEVICE_OBJECT,
        Name: *const u16,
        Unknown: *mut u8,
    ) -> NTSTATUS;

    pub fn PcAddAdapterDevice(
        DriverObject: PDRIVER_OBJECT,
        PhysicalDeviceObject: PDEVICE_OBJECT,
        StartDevice: Option<unsafe extern "C" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS>,
        MaxObjects: u32,
        DeviceExtensionSize: u32,
    ) -> NTSTATUS;
}

// ============================================================================
// Global Driver State
// ============================================================================

#[no_mangle]
pub static mut CONTROL_DEVICE_OBJECT: *mut DEVICE_OBJECT = core::ptr::null_mut();

static mut ETW_REG_HANDLE: u64 = 0;

/// Leyline Audio Driver ETW Provider GUID: {71549463-5E1E-4B7E-9F93-A65606E50D64}
const ETW_PROVIDER_GUID: GUID = GUID {
    Data1: 0x71549463,
    Data2: 0x5E1E,
    Data3: 0x4B7E,
    Data4: [0x9F, 0x93, 0xA6, 0x56, 0x06, 0xE5, 0x0D, 0x64],
};

// ============================================================================
// Driver Entry Point
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn DriverEntry(
    driver_object: PDRIVER_OBJECT,
    registry_path: PUNICODE_STRING,
) -> NTSTATUS {
    DbgPrint("Leyline: DriverEntry\n\0".as_ptr() as *const i8);

    // Register ETW Provider
    let _ = EtwRegister(
        &ETW_PROVIDER_GUID,
        None,
        core::ptr::null_mut(),
        &raw mut ETW_REG_HANDLE,
    );

    (*driver_object).DriverUnload = Some(DriverUnload);

    let status = PcInitializeAdapterDriver(driver_object, registry_path, Some(AddDevice));
    if status == STATUS_SUCCESS {
        DbgPrint("Leyline: PcInitializeAdapterDriver Success\n\0".as_ptr() as *const i8);

        // Safe CDO Dispatch Hooking
        ORIGINAL_DISPATCH_CREATE = (*driver_object).MajorFunction[IRP_MJ_CREATE as usize];
        (*driver_object).MajorFunction[IRP_MJ_CREATE as usize] = Some(dispatch_create);

        ORIGINAL_DISPATCH_CLOSE = (*driver_object).MajorFunction[IRP_MJ_CLOSE as usize];
        (*driver_object).MajorFunction[IRP_MJ_CLOSE as usize] = Some(dispatch_close);

        ORIGINAL_DISPATCH_CONTROL = (*driver_object).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize];
        (*driver_object).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
            Some(dispatch_device_control);
    }
    status
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn DriverUnload(_driver_object: PDRIVER_OBJECT) {
    DbgPrint("Leyline: DriverUnload\n\0".as_ptr() as *const i8);
    if ETW_REG_HANDLE != 0 {
        let _ = EtwUnregister(ETW_REG_HANDLE);
        ETW_REG_HANDLE = 0;
    }
}

// ============================================================================
// Stream Callbacks (Bridge to stream.rs)
// ============================================================================

pub unsafe extern "system" fn stream_query_interface(
    this: *mut u8,
    iid: *const GUID,
    out: *mut *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    if is_equal_guid(iid, &IID_IMiniportWaveRTStream) || is_equal_guid(iid, &IID_IUnknown) {
        (*com_obj).ref_count += 1;
        *out = this;
        STATUS_SUCCESS
    } else {
        *out = core::ptr::null_mut();
        STATUS_NOINTERFACE
    }
}

pub unsafe extern "system" fn stream_add_ref(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    (*com_obj).ref_count += 1;
    (*com_obj).ref_count
}

pub unsafe extern "system" fn stream_release(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    (*com_obj).ref_count -= 1;
    let count = (*com_obj).ref_count;
    if count == 0 {
        drop(alloc::boxed::Box::from_raw(com_obj));
    }
    count
}

pub unsafe extern "system" fn stream_set_format(_this: *mut u8, _format: *mut u8) -> NTSTATUS {
    STATUS_SUCCESS
}

pub unsafe extern "system" fn stream_set_state(this: *mut u8, state: i32) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    (*(*com_obj).stream).set_state(state)
}

pub unsafe extern "system" fn stream_get_position(this: *mut u8, position: *mut u64) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    (*(*com_obj).stream).get_position(position)
}

pub unsafe extern "system" fn stream_allocate_audio_buffer(
    this: *mut u8,
    req_size: usize,
    mdl: *mut *mut u8,
    act_size: *mut usize,
    off: *mut u32,
    cache: *mut i32,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    let status = (*(*com_obj).stream).allocate_audio_buffer(req_size, mdl as *mut PMDL);
    if status == STATUS_SUCCESS {
        if !act_size.is_null() {
            *act_size = req_size;
        }
        if !off.is_null() {
            *off = 0;
        }
        if !cache.is_null() {
            *cache = 1;
        }
    }
    status
}

pub unsafe extern "system" fn stream_free_audio_buffer(
    _this: *mut u8,
    _mdl: *mut u8,
    _size: usize,
) {
}
pub unsafe extern "system" fn stream_get_hw_latency(this: *mut u8, latency: *mut u32) {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    (*(*com_obj).stream).get_hw_latency(latency);
}
pub unsafe extern "system" fn stream_get_position_register(
    _this: *mut u8,
    _reg: *mut u8,
) -> NTSTATUS {
    0xC00000BBu32 as i32
}
pub unsafe extern "system" fn stream_get_clock_register(_this: *mut u8, _reg: *mut u8) -> NTSTATUS {
    0xC00000BBu32 as i32
}

pub fn is_equal_guid(a: *const GUID, b: &GUID) -> bool {
    unsafe {
        (*a).Data1 == b.Data1
            && (*a).Data2 == b.Data2
            && (*a).Data3 == b.Data3
            && (*a).Data4 == b.Data4
    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
