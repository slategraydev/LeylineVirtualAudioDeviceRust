// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

use wdk_sys::ntddk::*;
use wdk_sys::*;

// Globals for original dispatchers (wired in DriverEntry)
pub static mut ORIGINAL_DISPATCH_CREATE: PDRIVER_DISPATCH = None;
pub static mut ORIGINAL_DISPATCH_CLOSE: PDRIVER_DISPATCH = None;
pub static mut ORIGINAL_DISPATCH_CONTROL: PDRIVER_DISPATCH = None;

// External pointer to CDO defined in lib.rs
extern "C" {
    pub static mut CONTROL_DEVICE_OBJECT: *mut DEVICE_OBJECT;
}

// ============================================================================
// Dispatch Routines
// ============================================================================

pub unsafe extern "C" fn dispatch_create(device_object: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    if device_object == CONTROL_DEVICE_OBJECT {
        DbgPrint("Leyline: CDO Create\n\0".as_ptr() as *const i8);
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*irp).IoStatus.Information = 0;
        IofCompleteRequest(irp, 0);
        return STATUS_SUCCESS;
    }
    if let Some(original) = ORIGINAL_DISPATCH_CREATE {
        return original(device_object, irp);
    }
    STATUS_DEVICE_NOT_READY
}

pub unsafe extern "C" fn dispatch_close(device_object: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    if device_object == CONTROL_DEVICE_OBJECT {
        DbgPrint("Leyline: CDO Close\n\0".as_ptr() as *const i8);
        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*irp).IoStatus.Information = 0;
        IofCompleteRequest(irp, 0);
        return STATUS_SUCCESS;
    }
    if let Some(original) = ORIGINAL_DISPATCH_CLOSE {
        return original(device_object, irp);
    }
    STATUS_DEVICE_NOT_READY
}

pub unsafe extern "C" fn dispatch_device_control(
    device_object: PDEVICE_OBJECT,
    irp: PIRP,
) -> NTSTATUS {
    if device_object == CONTROL_DEVICE_OBJECT {
        let stack = (*irp)
            .Tail
            .Overlay
            .__bindgen_anon_2
            .__bindgen_anon_1
            .CurrentStackLocation;
        let ioctl = (*stack).Parameters.DeviceIoControl.IoControlCode;

        DbgPrint("Leyline: CDO IOCTL 0x%08X\n\0".as_ptr() as *const i8, ioctl);

        // TODO: Handle Leyline IOCTLs here

        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*irp).IoStatus.Information = 0;
        IofCompleteRequest(irp, 0);
        return STATUS_SUCCESS;
    }
    if let Some(original) = ORIGINAL_DISPATCH_CONTROL {
        return original(device_object, irp);
    }
    STATUS_DEVICE_NOT_READY
}
