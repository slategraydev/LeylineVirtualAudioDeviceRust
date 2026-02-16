// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

use wdk_sys::ntddk::*;
use wdk_sys::*;

use crate::adapter::get_device_extension;
use leyline_shared::*;

// Globals for original dispatchers (wired in DriverEntry)
pub static mut ORIGINAL_DISPATCH_CREATE: PDRIVER_DISPATCH = None;
pub static mut ORIGINAL_DISPATCH_CLOSE: PDRIVER_DISPATCH = None;
pub static mut ORIGINAL_DISPATCH_CONTROL: PDRIVER_DISPATCH = None;

// External pointer to CDO defined in lib.rs
extern "C" {
    pub static mut CONTROL_DEVICE_OBJECT: *mut DEVICE_OBJECT;
    pub static mut FUNCTIONAL_DEVICE_OBJECT: *mut DEVICE_OBJECT;
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
        let mut status = STATUS_SUCCESS;
        let mut information = 0;

        DbgPrint("Leyline: CDO IOCTL 0x%08X\n\0".as_ptr() as *const i8, ioctl);

        match ioctl {
            IOCTL_LEYLINE_GET_STATUS => {
                let out_buffer_len = (*stack).Parameters.DeviceIoControl.OutputBufferLength;
                if out_buffer_len >= core::mem::size_of::<u32>() as u32 {
                    let out_buffer = (*irp).AssociatedIrp.SystemBuffer as *mut u32;
                    *out_buffer = 0x1337BEEF; // Active Status Code
                    information = core::mem::size_of::<u32>();
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }
            }
            IOCTL_LEYLINE_MAP_BUFFER => {
                let out_buffer_len = (*stack).Parameters.DeviceIoControl.OutputBufferLength;
                if out_buffer_len >= core::mem::size_of::<usize>() as u32 {
                    if !FUNCTIONAL_DEVICE_OBJECT.is_null() {
                        let dev_ext = get_device_extension(FUNCTIONAL_DEVICE_OBJECT);
                        let out_buffer = (*irp).AssociatedIrp.SystemBuffer as *mut *mut u8;
                        *out_buffer = (*dev_ext).user_mapping;
                        information = core::mem::size_of::<usize>();
                    } else {
                        status = STATUS_DEVICE_NOT_READY;
                    }
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }
            }
            IOCTL_LEYLINE_MAP_PARAMS => {
                let out_buffer_len = (*stack).Parameters.DeviceIoControl.OutputBufferLength;
                if out_buffer_len >= core::mem::size_of::<usize>() as u32 {
                    if !FUNCTIONAL_DEVICE_OBJECT.is_null() {
                        let dev_ext = get_device_extension(FUNCTIONAL_DEVICE_OBJECT);
                        let out_buffer = (*irp).AssociatedIrp.SystemBuffer as *mut *mut u8;
                        *out_buffer = (*dev_ext).shared_params_user_mapping;
                        information = core::mem::size_of::<usize>();
                    } else {
                        status = STATUS_DEVICE_NOT_READY;
                    }
                } else {
                    status = STATUS_BUFFER_TOO_SMALL;
                }
            }
            _ => {
                status = STATUS_INVALID_DEVICE_REQUEST;
            }
        }
        (*irp).IoStatus.__bindgen_anon_1.Status = status;
        (*irp).IoStatus.Information = information as u64;
        IofCompleteRequest(irp, 0);
        return status;
    }
    if let Some(original) = ORIGINAL_DISPATCH_CONTROL {
        return original(device_object, irp);
    }
    STATUS_DEVICE_NOT_READY
}
