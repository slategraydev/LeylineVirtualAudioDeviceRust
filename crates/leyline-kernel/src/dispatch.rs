// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// DISPATCH HANDLERS
// IRP dispatch routines for the Control Device Object (CDO).
// Will be replaced with WDF CDO in Phase 4 of the ACX migration.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use wdk_sys::ntddk::*;
use wdk_sys::*;

use leyline_shared::*;

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// DISPATCH ROUTINES
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Handle IRP_MJ_DEVICE_CONTROL requests on the CDO.
///
/// # Safety
/// Standard kernel dispatch routine. Parameters must be valid pointers.
pub unsafe extern "C" fn dispatch_device_control(
    device_object: PDEVICE_OBJECT,
    irp: PIRP,
) -> NTSTATUS {
    let _ = device_object; // CDO validation will be added via WDF CDO in Phase 4.

    let stack = (*irp)
        .Tail
        .Overlay
        .__bindgen_anon_2
        .__bindgen_anon_1
        .CurrentStackLocation;
    let ioctl = (*stack).Parameters.DeviceIoControl.IoControlCode;
    let mut status = STATUS_SUCCESS;
    let mut information = 0;

    DbgPrint(c"Leyline: CDO IOCTL 0x%08X\n".as_ptr(), ioctl);

    match ioctl {
        IOCTL_LEYLINE_GET_STATUS => {
            let out_buffer_len = (*stack).Parameters.DeviceIoControl.OutputBufferLength;
            if out_buffer_len >= size_of::<u32>() as u32 {
                let out_buffer = (*irp).AssociatedIrp.SystemBuffer as *mut u32;
                *out_buffer = 0x1337BEEF; // Active Status Code.
                information = size_of::<u32>();
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

    status
}

use core::mem::size_of;
