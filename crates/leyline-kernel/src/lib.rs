// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// LEYLINE KERNEL CORE
// The entry point and global orchestration for the ACX audio driver.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#![no_std]

extern crate alloc;

pub mod adapter;
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

    // ---------------------------------------------------------------
    // KMDF + ACX Initialization Sequence
    // ---------------------------------------------------------------
    // TODO: The following calls require bindgen-generated types from
    // wdf.h and acx.h. They are stubbed until build.rs successfully
    // generates those bindings.
    //
    // 1. WDF_DRIVER_CONFIG_INIT(&driver_config, evt_driver_device_add)
    // 2. WdfDriverCreate(driver_object, registry_path, attrs, &config, &driver)
    // 3. ACX_DRIVER_CONFIG_INIT(&acx_config)
    // 4. AcxDriverInitialize(driver, &acx_config)

    let _ = driver_object;
    let _ = registry_path;

    DbgPrint(c"Leyline: ACX DriverEntry stub complete\n".as_ptr());
    STATUS_SUCCESS
}

/// KMDF EvtDriverUnload callback.
///
/// # Safety
/// Standard kernel DriverUnload callback. In KMDF, WDF handles most cleanup.
pub unsafe extern "C" fn driver_unload(_driver_object: *mut DRIVER_OBJECT) {
    DbgPrint(c"Leyline: DriverUnload\n".as_ptr());

    if ETW_REG_HANDLE != 0 {
        let _ = EtwUnregister(ETW_REG_HANDLE);
        ETW_REG_HANDLE = 0;
    }
}
