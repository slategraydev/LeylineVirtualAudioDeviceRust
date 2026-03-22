// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// DISPATCH HANDLERS
// WDF CDO (Control Device Object) for IOCTL handling.
// Replaces the legacy raw IoCreateDevice approach with
// WdfControlDeviceInitAllocate / WdfDeviceCreate.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use core::mem::size_of;
use core::ptr::null_mut;

use wdk_sys::ntddk::*;
use wdk_sys::*;

use leyline_shared::*;

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// CDO DEVICE NAME
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// NT device path for the CDO.
const CDO_DEVICE_NAME: &[u16] = &[
    b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
    b'e' as u16, b'\\' as u16, b'L' as u16, b'e' as u16, b'y' as u16, b'l' as u16,
    b'i' as u16, b'n' as u16, b'e' as u16, b'C' as u16, b'D' as u16, b'O' as u16,
    0u16,
];

/// Symbolic link for user-mode access.
const CDO_SYMLINK: &[u16] = &[
    b'\\' as u16, b'D' as u16, b'o' as u16, b's' as u16, b'D' as u16, b'e' as u16,
    b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b's' as u16, b'\\' as u16,
    b'L' as u16, b'e' as u16, b'y' as u16, b'l' as u16, b'i' as u16, b'n' as u16,
    b'e' as u16, b'C' as u16, b'D' as u16, b'O' as u16, 0u16,
];

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// CDO CREATION
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Create the WDF Control Device Object for user-mode IOCTL communication.
///
/// This replaces the legacy `IoCreateDevice` + `IoCreateSymbolicLink` pattern
/// with the WDF-managed `WdfControlDeviceInitAllocate` + `WdfDeviceCreate`
/// + `WdfDeviceCreateSymbolicLink` + `WdfControlFinishInitializing` flow.
///
/// # Safety
/// Must be called after WdfDriverCreate succeeds. `driver` must be valid.
pub unsafe fn create_control_device(driver: WDFDRIVER) -> NTSTATUS {
    DbgPrint(c"Leyline [CDO]: Creating WDF Control Device\n".as_ptr());

    // 1. Build the device name as a UNICODE_STRING
    let mut device_name = UNICODE_STRING {
        Length: ((CDO_DEVICE_NAME.len() - 1) * 2) as u16,
        MaximumLength: (CDO_DEVICE_NAME.len() * 2) as u16,
        Buffer: CDO_DEVICE_NAME.as_ptr() as *mut u16,
    };

    // 2. Get a SDDL string for unrestricted access (D:P(A;;GA;;;WD))
    //    This allows all users to open the CDO. Adjust for production.
    let sddl: &[u16] = &[
        b'D' as u16, b':' as u16, b'P' as u16, b'(' as u16, b'A' as u16,
        b';' as u16, b';' as u16, b'G' as u16, b'A' as u16, b';' as u16,
        b';' as u16, b';' as u16, b'W' as u16, b'D' as u16, b')' as u16,
        0u16,
    ];
    let sddl_string = UNICODE_STRING {
        Length: ((sddl.len() - 1) * 2) as u16,
        MaximumLength: (sddl.len() * 2) as u16,
        Buffer: sddl.as_ptr() as *mut u16,
    };

    // 3. Allocate CDO init structure
    let mut cdo_init = wdk_sys::call_unsafe_wdf_function_binding!(
        WdfControlDeviceInitAllocate,
        driver,
        &sddl_string as *const UNICODE_STRING as *const _
    );
    if cdo_init.is_null() {
        DbgPrint(c"Leyline [CDO]: WdfControlDeviceInitAllocate failed\n".as_ptr());
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // 4. Assign a device name
    let status = wdk_sys::call_unsafe_wdf_function_binding!(
        WdfDeviceInitAssignName,
        cdo_init,
        &mut device_name as *mut UNICODE_STRING as *mut _
    );
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [CDO]: WdfDeviceInitAssignName failed %x\n".as_ptr(), status);
        return status;
    }

    // 5. Register the IOCTL dispatch callback
    let mut io_queue_config: WDF_IO_QUEUE_CONFIG = core::mem::zeroed();
    io_queue_config.Size = core::mem::size_of::<WDF_IO_QUEUE_CONFIG>() as u32;
    io_queue_config.DispatchType = _WDF_IO_QUEUE_DISPATCH_TYPE::WdfIoQueueDispatchSequential;
    io_queue_config.PowerManaged = _WDF_TRI_STATE::WdfFalse;
    io_queue_config.EvtIoDeviceControl = Some(evt_io_device_control);

    // 6. Create the WDF device
    let mut cdo_attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
    cdo_attributes.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;

    let mut cdo_handle: WDFDEVICE = null_mut();
    let status = wdk_sys::call_unsafe_wdf_function_binding!(
        WdfDeviceCreate,
        &mut cdo_init,
        &mut cdo_attributes,
        &mut cdo_handle
    );
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [CDO]: WdfDeviceCreate failed %x\n".as_ptr(), status);
        return status;
    }

    // 7. Create the default I/O queue
    let mut queue_handle: WDFQUEUE = null_mut();
    let status = wdk_sys::call_unsafe_wdf_function_binding!(
        WdfIoQueueCreate,
        cdo_handle,
        &mut io_queue_config,
        &mut cdo_attributes,
        &mut queue_handle
    );
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [CDO]: WdfIoQueueCreate failed %x\n".as_ptr(), status);
        return status;
    }

    // 8. Create symbolic link
    let mut symlink_name = UNICODE_STRING {
        Length: ((CDO_SYMLINK.len() - 1) * 2) as u16,
        MaximumLength: (CDO_SYMLINK.len() * 2) as u16,
        Buffer: CDO_SYMLINK.as_ptr() as *mut u16,
    };
    let status = wdk_sys::call_unsafe_wdf_function_binding!(
        WdfDeviceCreateSymbolicLink,
        cdo_handle,
        &mut symlink_name as *mut UNICODE_STRING as *mut _
    );
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [CDO]: WdfDeviceCreateSymbolicLink failed %x\n".as_ptr(), status);
        return status;
    }

    // 9. Finish initialization — the CDO is now visible to user-mode.
    wdk_sys::call_unsafe_wdf_function_binding!(
        WdfControlFinishInitializing,
        cdo_handle
    );

    DbgPrint(c"Leyline [CDO]: Control Device created successfully\n".as_ptr());
    STATUS_SUCCESS
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// IOCTL DISPATCH
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// WDF EvtIoDeviceControl callback for CDO IOCTL handling.
///
/// Supported IOCTLs:
/// - `IOCTL_LEYLINE_GET_STATUS`: Returns a 32-bit status code.
/// - `IOCTL_LEYLINE_MAP_BUFFER`: Maps the shared audio buffer (stub).
/// - `IOCTL_LEYLINE_MAP_PARAMS`: Maps the shared parameter block (stub).
///
/// # Safety
/// Standard WDF callback. Parameters are OS-provided.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_io_device_control(
    _queue: WDFQUEUE,
    request: WDFREQUEST,
    _output_buffer_length: usize,
    _input_buffer_length: usize,
    io_control_code: u32,
) {
    DbgPrint(c"Leyline [CDO]: IOCTL 0x%08X\n".as_ptr(), io_control_code);

    let mut status = STATUS_SUCCESS;
    let mut information: usize = 0;

    match io_control_code {
        IOCTL_LEYLINE_GET_STATUS => {
            // Return a 32-bit active status code.
            let mut out_buffer: *mut core::ffi::c_void = null_mut();
            let mut out_length: usize = 0;

            let buf_status = wdk_sys::call_unsafe_wdf_function_binding!(
                WdfRequestRetrieveOutputBuffer,
                request,
                size_of::<u32>(),
                &mut out_buffer,
                &mut out_length
            );

            if NT_SUCCESS(buf_status) && out_length >= size_of::<u32>() {
                *(out_buffer as *mut u32) = 0x1337BEEF; // Active Status Code
                information = size_of::<u32>();
            } else {
                status = STATUS_BUFFER_TOO_SMALL;
            }
        }
        IOCTL_LEYLINE_MAP_BUFFER => {
            // TODO: Map the shared audio buffer to user-space.
            DbgPrint(c"Leyline [CDO]: MAP_BUFFER not yet implemented\n".as_ptr());
            status = STATUS_NOT_IMPLEMENTED;
        }
        IOCTL_LEYLINE_MAP_PARAMS => {
            // TODO: Map the shared parameter block to user-space.
            DbgPrint(c"Leyline [CDO]: MAP_PARAMS not yet implemented\n".as_ptr());
            status = STATUS_NOT_IMPLEMENTED;
        }
        _ => {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    // Complete the request.
    wdk_sys::call_unsafe_wdf_function_binding!(
        WdfRequestCompleteWithInformation,
        request,
        status,
        information as u64
    );
}
