// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

#![no_std]

extern crate alloc;

pub use leyline_shared::buffer;
pub mod math {
    pub use leyline_shared::math::WaveRTMath;
}
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

    pub fn PcNewPort(OutPort: *mut *mut u8, ClassId: *const GUID) -> NTSTATUS;

    pub fn PcRegisterSubdevice(
        DeviceObject: PDEVICE_OBJECT,
        Name: *const u16,
        Unknown: *mut u8,
    ) -> NTSTATUS;
}

// ============================================================================
// PortCls GUIDs
// ============================================================================
// Standard GUIDs for PortCls interfaces and classes.

#[allow(non_upper_case_globals)]
pub const CLSID_PortWaveRT: GUID = GUID {
    Data1: 0xB4C1147F,
    Data2: 0x810A,
    Data3: 0x443B,
    Data4: [0x99, 0x88, 0x51, 0xB4, 0xCD, 0x8A, 0x85, 0x4C],
};

#[allow(non_upper_case_globals)]
pub const IID_IPortWaveRT: GUID = GUID {
    Data1: 0x58AD9DCE,
    Data2: 0xC24D,
    Data3: 0x11D2,
    Data4: [0xBD, 0x41, 0x00, 0xC0, 0x4F, 0x75, 0x49, 0x28],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportWaveRT: GUID = GUID {
    Data1: 0xF6647993,
    Data2: 0xBA13,
    Data3: 0x4081,
    Data4: [0x9F, 0xB2, 0xCC, 0xF7, 0x39, 0xF7, 0x9C, 0x6A],
};

#[allow(non_upper_case_globals)]
pub const CLSID_PortTopology: GUID = GUID {
    Data1: 0xB4C11471,
    Data2: 0x810A,
    Data3: 0x443B,
    Data4: [0x99, 0x88, 0x51, 0xB4, 0xCD, 0x8A, 0x85, 0x4C],
};

#[allow(non_upper_case_globals)]
pub const IID_IPortTopology: GUID = GUID {
    Data1: 0xB4C1147B,
    Data2: 0x810A,
    Data3: 0x443B,
    Data4: [0x99, 0x88, 0x51, 0xB4, 0xCD, 0x8A, 0x85, 0x4C],
};

#[allow(non_upper_case_globals)]
pub const IID_IUnknown: GUID = GUID {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

// ============================================================================
// COM VTable Definitions
// ============================================================================

#[allow(non_snake_case)]
#[repr(C)]
pub struct IUnknownVTable {
    pub QueryInterface:
        unsafe extern "system" fn(this: *mut u8, iid: *const GUID, out: *mut *mut u8) -> NTSTATUS,
    pub AddRef: unsafe extern "system" fn(this: *mut u8) -> u32,
    pub Release: unsafe extern "system" fn(this: *mut u8) -> u32,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IMiniportWaveRTVTable {
    pub base: IUnknownVTable,
    // IMiniport
    pub GetDescription:
        unsafe extern "system" fn(this: *mut u8, out_description: *mut u8) -> NTSTATUS,
    pub DataRangeIntersection: unsafe extern "system" fn(
        this: *mut u8,
        pin_id: u32,
        data_range: *mut u8,
        matching_data_range: *mut u8,
        data_format_cb: u32,
        data_format: *mut u8,
        actual_data_format_cb: *mut u32,
    ) -> NTSTATUS,
    // IMiniportWaveRT
    pub Init: unsafe extern "system" fn(
        this: *mut u8,
        unknown_adapter: *mut u8,
        resource_list: *mut u8,
        port: *mut u8,
    ) -> NTSTATUS,
    pub GetDeviceDescription:
        unsafe extern "system" fn(this: *mut u8, description: *mut u8) -> NTSTATUS,
    pub NewStream: unsafe extern "system" fn(
        this: *mut u8,
        stream: *mut *mut u8,
        port_stream: *mut u8,
        pin: u32,
        capture: bool,
        format: *mut u8,
    ) -> NTSTATUS,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IMiniportWaveRTStreamVTable {
    pub base: IUnknownVTable,
    // IMiniportWaveRTStream
    pub SetState: unsafe extern "system" fn(this: *mut u8, state: i32) -> NTSTATUS,
    pub GetPosition: unsafe extern "system" fn(this: *mut u8, position: *mut u64) -> NTSTATUS,
    pub AllocateAudioBuffer: unsafe extern "system" fn(
        this: *mut u8,
        requested_size: usize,
        audio_buffer_mdl: *mut *mut u8,
        actual_size: *mut usize,
        offset_from_start: *mut u32,
        cache_type: *mut i32,
    ) -> NTSTATUS,
    pub FreeAudioBuffer:
        unsafe extern "system" fn(this: *mut u8, audio_buffer_mdl: *mut u8, buffer_size: usize),
    pub GetHWLatency: unsafe extern "system" fn(this: *mut u8, latency: *mut u32),
    pub GetPositionRegister: unsafe extern "system" fn(this: *mut u8, position_register: *mut u8),
    pub GetClockRegister: unsafe extern "system" fn(this: *mut u8, clock_register: *mut u8),
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
// Miniport Stream COM Wrapper
// ============================================================================

#[repr(C)]
pub struct MiniportWaveRTStreamCom {
    pub vtable: *const IMiniportWaveRTStreamVTable,
    pub stream: *mut MiniportWaveRTStream,
    pub ref_count: u32,
}

impl MiniportWaveRTStreamCom {
    pub fn new(stream: *mut MiniportWaveRTStream) -> Box<Self> {
        Box::new(Self {
            vtable: &STREAM_VTABLE,
            stream,
            ref_count: 1,
        })
    }
}

unsafe extern "system" fn stream_query_interface(
    this: *mut u8,
    iid: *const GUID,
    out: *mut *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    if iid.is_null() || out.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if is_equal_guid(iid, &IID_IPortWaveRTStream) || is_equal_guid(iid, &IID_IUnknown) {
        (*com_obj).ref_count += 1;
        *out = this;
        STATUS_SUCCESS
    } else {
        *out = core::ptr::null_mut();
        STATUS_NOINTERFACE
    }
}

unsafe extern "system" fn stream_add_ref(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    (*com_obj).ref_count += 1;
    (*com_obj).ref_count
}

unsafe extern "system" fn stream_release(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    (*com_obj).ref_count -= 1;
    let count = (*com_obj).ref_count;
    if count == 0 {
        drop(Box::from_raw(com_obj));
    }
    count
}

unsafe extern "system" fn stream_set_state(this: *mut u8, state: i32) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    (*(*com_obj).stream).set_state(state)
}

unsafe extern "system" fn stream_get_position(this: *mut u8, position: *mut u64) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    (*(*com_obj).stream).get_position(position)
}

unsafe extern "system" fn stream_allocate_audio_buffer(
    this: *mut u8,
    requested_size: usize,
    audio_buffer_mdl: *mut *mut u8,
    actual_size: *mut usize,
    offset_from_start: *mut u32,
    cache_type: *mut i32,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    let status =
        (*(*com_obj).stream).allocate_audio_buffer(requested_size, audio_buffer_mdl as *mut PMDL);

    if status == STATUS_SUCCESS {
        if !actual_size.is_null() {
            *actual_size = requested_size;
        }
        if !offset_from_start.is_null() {
            *offset_from_start = 0;
        }
        if !cache_type.is_null() {
            *cache_type = 1; // MmCached
        }
    }
    status
}

unsafe extern "system" fn stream_free_audio_buffer(
    _this: *mut u8,
    _audio_buffer_mdl: *mut u8,
    _buffer_size: usize,
) {
    // The stream's Drop implementation handles MDL cleanup.
}

unsafe extern "system" fn stream_get_hw_latency(this: *mut u8, latency: *mut u32) {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    (*(*com_obj).stream).get_hw_latency(latency);
}

unsafe extern "system" fn stream_get_position_register(
    _this: *mut u8,
    _position_register: *mut u8,
) {
}

unsafe extern "system" fn stream_get_clock_register(_this: *mut u8, _clock_register: *mut u8) {}

static STREAM_VTABLE: IMiniportWaveRTStreamVTable = IMiniportWaveRTStreamVTable {
    base: IUnknownVTable {
        QueryInterface: stream_query_interface,
        AddRef: stream_add_ref,
        Release: stream_release,
    },
    SetState: stream_set_state,
    GetPosition: stream_get_position,
    AllocateAudioBuffer: stream_allocate_audio_buffer,
    FreeAudioBuffer: stream_free_audio_buffer,
    GetHWLatency: stream_get_hw_latency,
    GetPositionRegister: stream_get_position_register,
    GetClockRegister: stream_get_clock_register,
};

#[allow(non_upper_case_globals)]
pub const IID_IPortWaveRTStream: GUID = GUID {
    Data1: 0x1070F136,
    Data2: 0x47E2,
    Data3: 0x4B3A,
    Data4: [0x8B, 0x1A, 0x5C, 0xC4, 0x54, 0x15, 0x6A, 0x7F],
};

// ============================================================================
// Miniport COM Wrapper
// ============================================================================

#[repr(C)]
pub struct MiniportWaveRTCom {
    pub vtable: *const IMiniportWaveRTVTable,
    pub inner: MiniportWaveRT,
    pub ref_count: u32,
}

impl MiniportWaveRTCom {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            vtable: &MINIPORT_VTABLE,
            inner: MiniportWaveRT::new(),
            ref_count: 1,
        })
    }
}

unsafe extern "system" fn miniport_query_interface(
    this: *mut u8,
    iid: *const GUID,
    out: *mut *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTCom;
    if iid.is_null() || out.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if is_equal_guid(iid, &IID_IMiniportWaveRT)
        || is_equal_guid(iid, &IID_IUnknown)
        || is_equal_guid(iid, &IID_IMiniport)
    {
        (*com_obj).ref_count += 1;
        *out = this;
        STATUS_SUCCESS
    } else {
        *out = core::ptr::null_mut();
        STATUS_NOINTERFACE
    }
}

unsafe extern "system" fn miniport_add_ref(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportWaveRTCom;
    (*com_obj).ref_count += 1;
    (*com_obj).ref_count
}

unsafe extern "system" fn miniport_release(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportWaveRTCom;
    (*com_obj).ref_count -= 1;
    let count = (*com_obj).ref_count;
    if count == 0 {
        // SAFETY: The object was allocated via Box::new in MiniportWaveRTCom::new().
        // We reconstruct the box to let it drop.
        drop(Box::from_raw(com_obj));
    }
    count
}

unsafe extern "system" fn miniport_get_description(
    _this: *mut u8,
    _out_description: *mut u8,
) -> NTSTATUS {
    STATUS_SUCCESS
}

unsafe extern "system" fn miniport_data_range_intersection(
    _this: *mut u8,
    _pin_id: u32,
    _data_range: *mut u8,
    _matching_data_range: *mut u8,
    _data_format_cb: u32,
    _data_format: *mut u8,
    _actual_data_format_cb: *mut u32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

unsafe extern "system" fn miniport_init(
    this: *mut u8,
    unknown_adapter: *mut u8,
    resource_list: *mut u8,
    port: *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTCom;
    (*com_obj).inner.init(
        unknown_adapter as PVOID,
        resource_list as PVOID,
        port as PVOID,
    )
}

unsafe extern "system" fn miniport_get_device_description(
    this: *mut u8,
    description: *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTCom;
    (*com_obj)
        .inner
        .get_device_description(description as PDEVICE_DESCRIPTION)
}

unsafe extern "system" fn miniport_new_stream(
    this: *mut u8,
    stream: *mut *mut u8,
    _port_stream: *mut u8,
    pin: u32,
    capture: bool,
    format: *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTCom;
    let stream_ptr = (*com_obj).inner.new_stream(pin, capture, format as PVOID);

    if stream_ptr.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // We need to wrap the stream in a COM object as well.
    let stream_com = MiniportWaveRTStreamCom::new(stream_ptr);
    *stream = Box::into_raw(stream_com) as *mut u8;

    STATUS_SUCCESS
}

static MINIPORT_VTABLE: IMiniportWaveRTVTable = IMiniportWaveRTVTable {
    base: IUnknownVTable {
        QueryInterface: miniport_query_interface,
        AddRef: miniport_add_ref,
        Release: miniport_release,
    },
    GetDescription: miniport_get_description,
    DataRangeIntersection: miniport_data_range_intersection,
    Init: miniport_init,
    GetDeviceDescription: miniport_get_device_description,
    NewStream: miniport_new_stream,
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniport: GUID = GUID {
    Data1: 0xB4C11470,
    Data2: 0x810A,
    Data3: 0x443B,
    Data4: [0x99, 0x88, 0x51, 0xB4, 0xCD, 0x8A, 0x85, 0x4C],
};

fn is_equal_guid(a: *const GUID, b: &GUID) -> bool {
    unsafe {
        (*a).Data1 == b.Data1
            && (*a).Data2 == b.Data2
            && (*a).Data3 == b.Data3
            && (*a).Data4 == b.Data4
    }
}

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
    device_object: PDEVICE_OBJECT,
    _irp: PIRP,
    resource_list: PVOID,
) -> NTSTATUS {
    let mut status: NTSTATUS;

    // 1. Create the WaveRT Port object.
    let mut port: *mut u8 = core::ptr::null_mut();
    status = PcNewPort(&mut port, &CLSID_PortWaveRT);
    if status != STATUS_SUCCESS {
        return status;
    }

    // 2. Create the Miniport object.
    // The COM wrapper will manage the lifecycle of the MiniportWaveRT.
    let miniport_com = MiniportWaveRTCom::new();
    let miniport_ptr = Box::into_raw(miniport_com) as *mut u8;

    // 3. Initialize the Port with the Miniport.
    // We call the Init method from the IPort interface VTable (index 3).
    type PortInitFn = unsafe extern "system" fn(
        this: *mut u8,
        device_object: PDEVICE_OBJECT,
        unknown_adapter: PVOID,
        resource_list: PVOID,
        miniport: *mut u8,
    ) -> NTSTATUS;

    let vtable = *(port as *const *const *const u8);
    let init_ptr = *vtable.add(3);
    let init_fn: PortInitFn = core::mem::transmute(init_ptr);

    status = init_fn(
        port,
        device_object,
        core::ptr::null_mut(),
        resource_list,
        miniport_ptr,
    );
    if status != STATUS_SUCCESS {
        // In case of failure, we should ideally release the port and miniport.
        return status;
    }

    // 4. Register the WaveRT subdevice.
    // The name "Wave" is used by convention to identify the streaming filter.
    let wave_name: [u16; 5] = [0x0057, 0x0061, 0x0076, 0x0065, 0x0000]; // "Wave\0"
    status = PcRegisterSubdevice(device_object, wave_name.as_ptr(), port);

    status
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
