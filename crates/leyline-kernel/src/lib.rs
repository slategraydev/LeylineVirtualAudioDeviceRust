// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

#![allow(clippy::missing_transmute_annotations)]

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// LEYLINE KERNEL CORE
// The entry point and global orchestration for the ACX audio driver.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#![no_std]

extern crate alloc;

pub mod adapter;
pub mod circuit;
pub mod constants;
pub mod dispatch;
pub mod stream;

use core::ptr::null_mut;

use wdk_alloc::WdkAllocator;
use wdk_sys::ntddk::*;
use wdk_sys::*;
#[cfg(not(test))]
extern crate wdk_panic;

#[allow(clippy::all)]
#[allow(non_camel_case_types)]
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
#[allow(unused_imports)]
#[allow(dead_code)]
#[allow(clippy::unnecessary_cast)]
#[allow(clippy::useless_transmute)]
#[allow(clippy::too_many_arguments)]
#[allow(unnecessary_transmutes)]
pub mod audio {
    // Include generated bindings. Shadow copy used if environment lacks OUT_DIR.
    #[cfg(not(rust_analyzer))]
    include!(concat!(env!("OUT_DIR"), "/audio_bindings.rs"));

    #[cfg(rust_analyzer)]
    include!("audio_bindings.rs");
}

pub use audio as audio_bindings;

#[global_allocator]
static GLOBAL: WdkAllocator = WdkAllocator;

/// Required by acxstub.lib to verify matching ACX framework version at runtime.
/// We use ACX 1.1, so the minimum version required is minor version 1.
#[unsafe(no_mangle)]
pub static mut AcxMinimumVersionRequired: u32 = 1;

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// GLOBAL DRIVER STATE
// Persistent objects managed across the driver lifecycle.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

static mut ETW_REG_HANDLE: u64 = 0;

/// ETW Provider GUID: {71549463-5E1E-4B7E-9F93-A65606E50D64}
const ETW_PROVIDER_GUID: GUID = GUID {
    Data1: 0x71549463,
    Data2: 0x5E1E,
    Data3: 0x4B7E,
    Data4: [0x9F, 0x93, 0xA6, 0x56, 0x06, 0xE5, 0x0D, 0x64],
};

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// DRIVER ENTRY POINT
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Initialize the driver and register with KMDF and ACX.
///
/// # Safety
/// Standard kernel DriverEntry. Parameters must be OS-provided pointers.
#[unsafe(export_name = "DriverEntry")]
pub unsafe extern "system" fn driver_entry(
    driver_object: *mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    DbgPrint(c"Leyline: ACX DriverEntry v0.2.0\n".as_ptr());

    // Register ETW Provider for diagnostics.
    let _ = EtwRegister(
        &ETW_PROVIDER_GUID,
        None,
        null_mut(),
        &raw mut ETW_REG_HANDLE,
    );

    // 1. Initialize WDF_DRIVER_CONFIG
    let mut driver_config: wdk_sys::WDF_DRIVER_CONFIG = unsafe { core::mem::zeroed() };
    driver_config.Size = core::mem::size_of::<wdk_sys::WDF_DRIVER_CONFIG>() as u32;
    driver_config.EvtDriverDeviceAdd = Some(crate::adapter::evt_driver_device_add);
    driver_config.EvtDriverUnload = Some(driver_unload);

    // 2. Initialize WDF_OBJECT_ATTRIBUTES for the driver
    let mut driver_attributes: wdk_sys::WDF_OBJECT_ATTRIBUTES = unsafe { core::mem::zeroed() };
    driver_attributes.Size = core::mem::size_of::<wdk_sys::WDF_OBJECT_ATTRIBUTES>() as u32;

    // 3. Create WDFDRIVER
    let mut driver_handle: wdk_sys::WDFDRIVER = core::ptr::null_mut();
    let status = unsafe { 
        wdk_sys::call_unsafe_wdf_function_binding!(
            WdfDriverCreate,
            driver_object,
            registry_path,
            &mut driver_attributes,
            &mut driver_config,
            &mut driver_handle
        ) 
    };

    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline: WdfDriverCreate failed 0x%X\n".as_ptr(), status);
        return status;
    }

    // 4. Initialize ACX_DRIVER_CONFIG
    let mut acx_config: crate::audio_bindings::ACX_DRIVER_CONFIG = unsafe { core::mem::zeroed() };
    acx_config.Size = core::mem::size_of::<crate::audio_bindings::ACX_DRIVER_CONFIG>() as u32;

    // 5. Initialize ACX Driver
    let status = unsafe { 
        let func: crate::audio_bindings::PFN_ACXDRIVERINITIALIZE = core::mem::transmute(
            *(core::ptr::addr_of!(crate::audio_bindings::AcxFunctions) as *const _ as *const *const core::ffi::c_void)
                .add(crate::audio_bindings::_ACXFUNCENUM_AcxDriverInitializeTableIndex as usize)
        );
        func.unwrap()(crate::audio_bindings::AcxDriverGlobals, driver_handle as _, &mut acx_config)
    };

    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline: AcxDriverInitialize failed 0x%X\n".as_ptr(), status);
        return status;
    }

    // 6. Create WDF Control Device Object for user-mode IOCTLs
    let status = crate::dispatch::create_control_device(driver_handle);
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline: CDO creation failed 0x%X (non-fatal)\n".as_ptr(), status);
        // CDO failure is non-fatal — the audio path still works.
    }

    DbgPrint(c"Leyline: ACX DriverEntry complete\n".as_ptr());
    STATUS_SUCCESS
}

/// KMDF EvtDriverUnload callback.
///
/// # Safety
/// Standard kernel DriverUnload callback. In KMDF, WDF handles most cleanup.
pub unsafe extern "C" fn driver_unload(_driver_object: wdk_sys::WDFDRIVER) {
    DbgPrint(c"Leyline: DriverUnload\n".as_ptr());

    if ETW_REG_HANDLE != 0 {
        let _ = EtwUnregister(ETW_REG_HANDLE);
        ETW_REG_HANDLE = 0;
    }
}
