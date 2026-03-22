// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ADAPTER MANAGEMENT
// Logic for PnP orchestration and KMDF/ACX device initialization.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use wdk_sys::ntddk::*;
use wdk_sys::*;


#[repr(C)]
pub struct DeviceExtension {
    pub shared_params: *mut leyline_shared::SharedParameters,
    pub loopback_buffer: *mut u8,
    pub loopback_size: usize,
    // Future field: ACX Circuit pointers.
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// PNP CALLBACKS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// KMDF EvtDriverDeviceAdd callback for hardware initialization.
///
/// # Safety
/// Standard KMDF callback. Parameters must be OS-provided pointers.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_driver_device_add(
    _driver: WDFDRIVER,
    mut device_init: *mut wdk_sys::WDFDEVICE_INIT,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtDriverDeviceAdd\n".as_ptr());

    // 1. Initialize the ACX part of the WDFDEVICE_INIT
    let status = unsafe {
        let func: crate::audio_bindings::PFN_ACXDEVICEINITINITIALIZE = core::mem::transmute(
            *(core::ptr::addr_of!(crate::audio_bindings::AcxFunctions) as *const _ as *const *const core::ffi::c_void)
                .add(crate::audio_bindings::_ACXFUNCENUM_AcxDeviceInitInitializeTableIndex as usize)
        );
        let mut acx_device_init_config: crate::audio_bindings::ACX_DEVICEINIT_CONFIG = core::mem::zeroed();
        acx_device_init_config.Size = core::mem::size_of::<crate::audio_bindings::ACX_DEVICEINIT_CONFIG>() as u32;

        func.unwrap()(crate::audio_bindings::AcxDriverGlobals, device_init as *mut _, &mut acx_device_init_config)
    };
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [ACX]: AcxDeviceInitInitialize failed %x\n".as_ptr(), status);
        return status;
    }

    // 2. Setup WDF_OBJECT_ATTRIBUTES for the FDO
    let mut device_attributes: wdk_sys::WDF_OBJECT_ATTRIBUTES = unsafe { core::mem::zeroed() };
    device_attributes.Size = core::mem::size_of::<wdk_sys::WDF_OBJECT_ATTRIBUTES>() as u32;

    // 2.5 Setup PnP/Power callbacks
    let mut pnp_power_callbacks: wdk_sys::WDF_PNPPOWER_EVENT_CALLBACKS = unsafe { core::mem::zeroed() };
    pnp_power_callbacks.Size = core::mem::size_of::<wdk_sys::WDF_PNPPOWER_EVENT_CALLBACKS>() as u32;
    pnp_power_callbacks.EvtDevicePrepareHardware = Some(evt_device_prepare_hardware);
    pnp_power_callbacks.EvtDeviceReleaseHardware = Some(evt_device_release_hardware);

    unsafe {
        wdk_sys::call_unsafe_wdf_function_binding!(
            WdfDeviceInitSetPnpPowerEventCallbacks,
            device_init,
            &mut pnp_power_callbacks
        );
    }

    // 3. Create the WDFDEVICE
    let mut device_handle: wdk_sys::WDFDEVICE = core::ptr::null_mut();
    let status = unsafe { 
        wdk_sys::call_unsafe_wdf_function_binding!(
            WdfDeviceCreate,
            &mut device_init,
            &mut device_attributes,
            &mut device_handle
        ) 
    };
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [ACX]: WdfDeviceCreate failed %x\n".as_ptr(), status);
        return status;
    }

    // 4. Initialize the ACX device
    let status = unsafe {
        let func: crate::audio_bindings::PFN_ACXDEVICEINITIALIZE = core::mem::transmute(
            *(core::ptr::addr_of!(crate::audio_bindings::AcxFunctions) as *const _ as *const *const core::ffi::c_void)
                .add(crate::audio_bindings::_ACXFUNCENUM_AcxDeviceInitializeTableIndex as usize)
        );
        let mut acx_device_config: crate::audio_bindings::ACX_DEVICE_CONFIG = core::mem::zeroed();
        acx_device_config.Size = core::mem::size_of::<crate::audio_bindings::ACX_DEVICE_CONFIG>() as u32;

        func.unwrap()(crate::audio_bindings::AcxDriverGlobals, device_handle as _, &mut acx_device_config)
    };
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [ACX]: AcxDeviceInitialize failed %x\n".as_ptr(), status);
        return status;
    }

    // 5. Create AcxCircuit mappings 
    let status = crate::circuit::create_render_circuit(device_handle);
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [ACX]: create_render_circuit failed %x\n".as_ptr(), status);
        return status;
    }

    let status = crate::circuit::create_capture_circuit(device_handle);
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [ACX]: create_capture_circuit failed %x\n".as_ptr(), status);
        return status;
    }

    DbgPrint(c"Leyline [ACX]: Device and circuits added successfully\n".as_ptr());
    STATUS_SUCCESS
}

/// EvtDevicePrepareHardware callback
///
/// # Safety
/// Standard ACX callback, parameters managed by WDF.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_device_prepare_hardware(
    _device: WDFDEVICE,
    _resources_raw: WDFCMRESLIST,
    _resources_translated: WDFCMRESLIST,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtDevicePrepareHardware\n".as_ptr());
    STATUS_SUCCESS
}

/// EvtDeviceReleaseHardware callback
///
/// # Safety
/// Standard ACX callback, parameters managed by WDF.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_device_release_hardware(
    _device: WDFDEVICE,
    _resources_translated: WDFCMRESLIST,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtDeviceReleaseHardware\n".as_ptr());
    STATUS_SUCCESS
}

