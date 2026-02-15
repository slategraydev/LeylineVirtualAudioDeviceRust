// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

#![no_std]

extern crate alloc;

pub mod buffer;
pub mod math;
pub mod stream;

#[cfg(test)]
mod test_harness;

use crate::stream::MiniportWaveRTStream;
use alloc::boxed::Box;
use wdk_alloc::WDKAllocator;
use wdk_sys::ntddk::*;
use wdk_sys::*;

// Standard WDK types mapped for convenience.

// ============================================================================
// Constants & Configuration
// ============================================================================
// Driver-wide constants and default configuration values for the wave adapter.

/// Tag used for kernel memory allocations by this driver.
const _POOL_TAG: u32 = u32::from_be_bytes(*b"LLAD");

/// Maximum number of concurrent audio streams supported by the miniport.
const MAX_STREAMS: usize = 4;

/// Default size for the initial wave buffer (64 KB).
const _DEFAULT_BUFFER_SIZE: usize = 64 * 1024;

// ============================================================================
// Global Allocator
// ============================================================================
// The WDKAllocator provides safe heap allocation in kernel-mode,
// enabling the use of `Box`.

#[global_allocator]
static GLOBAL: WDKAllocator = WDKAllocator;

// ============================================================================
// Miniport Structure
// ============================================================================
// The core WaveRT miniport object, managing hardware resources and streams.

pub struct MiniportWaveRT {
    pub max_pci_bar: u32,
    pub is_initialized: bool,
    pub streams: [Option<Box<MiniportWaveRTStream>>; MAX_STREAMS],
}

// Global instance for prototype/demonstration purposes (Session #04).
// In a full implementation, this would be managed via DeviceExtension or PortCls.
pub static mut MINI_PORT: Option<MiniportWaveRT> = None;

pub static mut SHARED_PARAMS: leyline_shared::SharedParameters = leyline_shared::SharedParameters {
    master_gain_bits: 0x3F800000, // 1.0
    peak_l_bits: 0,
    peak_r_bits: 0,
};

// ============================================================================
// Miniport Implementation
// ============================================================================
// Logic for miniport lifecycle and stream management.

impl MiniportWaveRT {
    /// Creates a new, uninitialized instance of the miniport.
    pub fn new() -> Self {
        Self {
            max_pci_bar: 0,
            is_initialized: false,
            streams: [None, None, None, None],
        }
    }

    /// Initialize the miniport with hardware resources.
    pub fn init(
        &mut self,
        _unknown_adapter: PVOID,
        _resource_list: PVOID,
        _port: PVOID,
    ) -> NTSTATUS {
        // Future: Register interrupt handlers and map BAR resources.
        self.is_initialized = true;
        STATUS_SUCCESS
    }

    /// Create a new WaveRT stream for a specific pin.
    /// Returns a raw pointer to the stream object for PortCls.
    pub fn new_stream(
        &mut self,
        _pin: u32,
        _capture: bool,
        _data_format: PVOID,
    ) -> *mut MiniportWaveRTStream {
        if !self.is_initialized {
            return core::ptr::null_mut();
        }

        for stream_slot in self.streams.iter_mut() {
            if stream_slot.is_none() {
                // SAFETY: The stream corresponds to a kernel object whose lifecycle
                // is controlled by the port driver. Box ensures pointer stability.
                unsafe {
                    *stream_slot = Some(Box::new(MiniportWaveRTStream::new(
                        _data_format as PVOID,
                        _capture,
                        Box::new(crate::stream::KernelTimeSource),
                    )));
                }

                return stream_slot.as_mut().unwrap().as_mut() as *mut MiniportWaveRTStream;
            }
        }

        core::ptr::null_mut()
    }

    pub fn get_device_description(&self, _device_description: PDEVICE_DESCRIPTION) -> NTSTATUS {
        STATUS_SUCCESS
    }
}

// ============================================================================
// Driver Entry Point
// ============================================================================
// The initial entry point called by the Windows I/O manager.

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    driver_object: PDRIVER_OBJECT,
    _registry_path: PUNICODE_STRING,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut device_object: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut device_name = UNICODE_STRING::default();
    let mut symbolic_link = UNICODE_STRING::default();

    // {A5553592-2D2F-4A98-84B2-7A9B9355152F}
    let device_name_str = encode_unicode_string("\\Device\\LeylineAudio");
    let sym_link_str = encode_unicode_string("\\??\\LeylineAudio");

    RtlInitUnicodeString(&mut device_name, device_name_str.as_ptr());
    RtlInitUnicodeString(&mut symbolic_link, sym_link_str.as_ptr());

    // Create the Control Device Object
    status = IoCreateDevice(
        driver_object,
        0,
        &mut device_name,
        FILE_DEVICE_UNKNOWN,
        0,
        FALSE as u8,
        &mut device_object,
    );

    if status != STATUS_SUCCESS {
        return status;
    }

    // Create the symbolic link for user-space access
    status = IoCreateSymbolicLink(&mut symbolic_link, &mut device_name);
    if status != STATUS_SUCCESS {
        IoDeleteDevice(device_object);
        return status;
    }

    // Set up dispatch routines
    (*driver_object).MajorFunction[IRP_MJ_CREATE as usize] = Some(DispatchCreateClose);
    (*driver_object).MajorFunction[IRP_MJ_CLOSE as usize] = Some(DispatchCreateClose);
    (*driver_object).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(DispatchDeviceControl);

    status = STATUS_SUCCESS;
    status
}

#[no_mangle]
pub unsafe extern "C" fn DispatchCreateClose(
    _device_object: PDEVICE_OBJECT,
    irp: PIRP,
) -> NTSTATUS {
    (*irp).IoStatus.Information = 0;
    (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IofCompleteRequest(irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

fn encode_unicode_string(s: &str) -> alloc::vec::Vec<u16> {
    let mut v: alloc::vec::Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

#[no_mangle]
pub unsafe extern "C" fn DispatchDeviceControl(
    _device_object: PDEVICE_OBJECT,
    irp: PIRP,
) -> NTSTATUS {
    // Hoist local variables
    let irp_sp: PIO_STACK_LOCATION;
    let ioctl_code: u32;

    irp_sp = (*irp)
        .Tail
        .Overlay
        .__bindgen_anon_2
        .__bindgen_anon_1
        .CurrentStackLocation;
    ioctl_code = unsafe { (*irp_sp).Parameters.DeviceIoControl.IoControlCode };

    match ioctl_code {
        leyline_shared::IOCTL_LEYLINE_SET_CONFIG => {
            // Future: Handle buffer/latency parameter updates.
        }
        leyline_shared::IOCTL_LEYLINE_GET_STATUS => {
            // Future: Report stream health and errors.
        }
        leyline_shared::IOCTL_LEYLINE_MAP_BUFFER => {
            // Logic to return the user-space pointer for the first active stream.
            let output_buffer = (*irp).AssociatedIrp.SystemBuffer;
            let output_length = (*irp_sp).Parameters.DeviceIoControl.OutputBufferLength;

            if output_length >= core::mem::size_of::<*mut u8>() as u32 && !output_buffer.is_null() {
                unsafe {
                    if let Some(ref mut miniport) = MINI_PORT {
                        if let Some(ref mut stream) = miniport.streams[0] {
                            let user_ptr = stream.map_user_buffer(core::ptr::null_mut());
                            *(output_buffer as *mut *mut u8) = user_ptr;
                            (*irp).IoStatus.Information = core::mem::size_of::<*mut u8>() as u64;
                            (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                        } else {
                            (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_FOUND;
                        }
                    } else {
                        (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_NOT_READY;
                    }
                }
            } else {
                (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
            }
        }
        leyline_shared::IOCTL_LEYLINE_MAP_PARAMS => {
            let output_buffer = (*irp).AssociatedIrp.SystemBuffer;
            let output_length = (*irp_sp).Parameters.DeviceIoControl.OutputBufferLength;

            if output_length >= core::mem::size_of::<*mut u8>() as u32 && !output_buffer.is_null() {
                unsafe {
                    // For this prototype, we return the kernel address which is mapped
                    // to user-space via a separate process or by being in the correct
                    // context. However, MmMapLockedPagesSpecifyCache is better.
                    // For now, we'll assume the params are in a globally accessible region
                    // or we'll simplify by using a direct pointer for this prototype.
                    *(output_buffer as *mut *mut leyline_shared::SharedParameters) =
                        &raw mut SHARED_PARAMS;
                    (*irp).IoStatus.Information = core::mem::size_of::<*mut u8>() as u64;
                    (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                }
            } else {
                (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_BUFFER_TOO_SMALL;
            }
        }
        _ => unsafe {
            (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
        },
    }

    IofCompleteRequest(irp, IO_NO_INCREMENT as i8);
    STATUS_SUCCESS
}

// ============================================================================
// Panic Handler
// ============================================================================
// Required for no_std environments to handle critical runtime errors.

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
