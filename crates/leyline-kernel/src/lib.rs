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

use crate::stream::MiniportWaveRTStream;
use crate::stream::TimeSource;
use alloc::boxed::Box;
use wdk_alloc::WDKAllocator;
use wdk_sys::*;

// ============================================================================
// PortCls External Declarations
// ============================================================================
// These functions are exported by portcls.sys but are not always present
// in the default wdk-sys bindings.

extern "system" {
    pub fn PcAddAdapterDevice(
        DriverObject: PDRIVER_OBJECT,
        PhysicalDeviceObject: PDEVICE_OBJECT,
        StartDevice: Option<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS>,
        MaxOutputStreams: u32,
        DeviceExtensionSize: u32,
    ) -> NTSTATUS;

    pub fn PcInitializeAdapterDriver(
        DriverObject: PDRIVER_OBJECT,
        RegistryPath: PUNICODE_STRING,
        AddDevice: Option<unsafe extern "C" fn(PDRIVER_OBJECT, PDEVICE_OBJECT) -> NTSTATUS>,
    ) -> NTSTATUS;
}

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
                        Box::new(crate::stream::KernelTimeSource) as Box<dyn TimeSource>,
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
    registry_path: PUNICODE_STRING,
) -> NTSTATUS {
    // PortCls drivers use PcInitializeAdapterDriver to set up the driver object
    // with the appropriate dispatch and AddDevice routines.
    PcInitializeAdapterDriver(driver_object, registry_path, Some(AddDevice))
}

#[no_mangle]
pub unsafe extern "C" fn AddDevice(
    driver_object: PDRIVER_OBJECT,
    physical_device_object: PDEVICE_OBJECT,
) -> NTSTATUS {
    // PcAddAdapterDevice creates the FDO and handles device lifecycle.
    PcAddAdapterDevice(
        driver_object,
        physical_device_object,
        Some(StartDevice),
        MAX_STREAMS as u32,
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn StartDevice(
    _device_object: PDEVICE_OBJECT,
    _irp: PIRP,
    _resource_list: PVOID,
) -> NTSTATUS {
    // This is called when the OS starts the audio device.
    // In a WaveRT driver, this is where we would normally register
    // the Wave and Topology filters.
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
