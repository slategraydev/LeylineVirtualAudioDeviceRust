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
use crate::stream::*;
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

    pub fn PcRegisterPhysicalConnection(
        DeviceObject: PDEVICE_OBJECT,
        FromUnknown: *mut u8,
        FromPin: u32,
        ToUnknown: *mut u8,
        ToPin: u32,
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
pub const IID_IMiniportTopology: GUID = GUID {
    Data1: 0xB4C11479,
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

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_TYPE_AUDIO: GUID = GUID {
    Data1: 0x73647561,
    Data2: 0x0000,
    Data3: 0x0010,
    Data4: [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
};

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_SUBTYPE_PCM: GUID = GUID {
    Data1: 0x00000001,
    Data2: 0x0000,
    Data3: 0x0010,
    Data4: [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
};

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_SUBTYPE_IEEE_FLOAT: GUID = GUID {
    Data1: 0x00000003,
    Data2: 0x0000,
    Data3: 0x0010,
    Data4: [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
};

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_SPECIFIER_WAVEFORMATEX: GUID = GUID {
    Data1: 0x05589F81,
    Data2: 0xC356,
    Data3: 0x11CE,
    Data4: [0xBF, 0x01, 0x00, 0xAA, 0x00, 0x55, 0x59, 0x5A],
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
pub struct IMiniportTopologyVTable {
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
    // IMiniportTopology
    pub Init: unsafe extern "system" fn(
        this: *mut u8,
        unknown_adapter: *mut u8,
        resource_list: *mut u8,
        port: *mut u8,
    ) -> NTSTATUS,
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
// Miniport Topology COM Wrapper
// ============================================================================

#[repr(C)]
pub struct MiniportTopologyCom {
    pub vtable: *const IMiniportTopologyVTable,
    pub inner: MiniportTopology,
    pub ref_count: u32,
}

impl MiniportTopologyCom {
    pub fn new(is_capture: bool) -> Box<Self> {
        Box::new(Self {
            vtable: &TOPOLOGY_VTABLE,
            inner: MiniportTopology::new(is_capture),
            ref_count: 1,
        })
    }
}

unsafe extern "system" fn topology_query_interface(
    this: *mut u8,
    iid: *const GUID,
    out: *mut *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportTopologyCom;
    if iid.is_null() || out.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if is_equal_guid(iid, &IID_IMiniportTopology)
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

unsafe extern "system" fn topology_add_ref(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportTopologyCom;
    (*com_obj).ref_count += 1;
    (*com_obj).ref_count
}

unsafe extern "system" fn topology_release(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportTopologyCom;
    (*com_obj).ref_count -= 1;
    let count = (*com_obj).ref_count;
    if count == 0 {
        // SAFETY: Reconstruct the box to drop the memory.
        drop(Box::from_raw(com_obj));
    }
    count
}

unsafe extern "system" fn topology_get_description(
    this: *mut u8,
    out_description: *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportTopologyCom;
    let description = out_description as *mut *const PCFILTER_DESCRIPTOR;

    if (*com_obj).inner.is_capture {
        *description = &TOPO_CAPTURE_FILTER_DESCRIPTOR;
    } else {
        *description = &TOPO_RENDER_FILTER_DESCRIPTOR;
    }
    STATUS_SUCCESS
}

unsafe extern "system" fn topology_data_range_intersection(
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

unsafe extern "system" fn topology_init(
    this: *mut u8,
    unknown_adapter: *mut u8,
    resource_list: *mut u8,
    port: *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportTopologyCom;
    (*com_obj).inner.init(
        unknown_adapter as PVOID,
        resource_list as PVOID,
        port as PVOID,
    )
}

static TOPOLOGY_VTABLE: IMiniportTopologyVTable = IMiniportTopologyVTable {
    base: IUnknownVTable {
        QueryInterface: topology_query_interface,
        AddRef: topology_add_ref,
        Release: topology_release,
    },
    GetDescription: topology_get_description,
    DataRangeIntersection: topology_data_range_intersection,
    Init: topology_init,
};

// ============================================================================
// Miniport Topology Structure
// ============================================================================

pub struct MiniportTopology {
    pub is_initialized: bool,
    pub is_capture: bool,
}

impl MiniportTopology {
    pub fn new(is_capture: bool) -> Self {
        Self {
            is_initialized: false,
            is_capture,
        }
    }

    pub fn init(
        &mut self,
        _unknown_adapter: PVOID,
        _resource_list: PVOID,
        _port: PVOID,
    ) -> NTSTATUS {
        self.is_initialized = true;
        STATUS_SUCCESS
    }
}

// ============================================================================
// PortCls Topology Structs
// ============================================================================

#[allow(non_snake_case)]
#[repr(C)]
pub struct PCCONNECTION {
    pub FromNode: u32,
    pub FromPin: u32,
    pub ToNode: u32,
    pub ToPin: u32,
}

pub const PCFILTER_NODE: u32 = !0u32;

// ============================================================================
// Pin IDs
// ============================================================================

const KSPIN_WAVE_SINK: u32 = 0;
const KSPIN_WAVE_BRIDGE: u32 = 1;

const KSPIN_TOPO_BRIDGE: u32 = 0;
const KSPIN_TOPO_LINEOUT: u32 = 1;

// ============================================================================
// Wave Filter Description (Static)
// ============================================================================

#[repr(transparent)]
struct SyncPtr<T>(*const T);
unsafe impl<T> Sync for SyncPtr<T> {}

static WAVE_DATARANGES: [SyncPtr<KSDATARANGE>; 2] = [
    SyncPtr(&PCM_DATARANGE.DataRange as *const KSDATARANGE),
    SyncPtr(&FLOAT_DATARANGE.DataRange as *const KSDATARANGE),
];

static PCM_DATARANGE: KSDATARANGE_AUDIO = KSDATARANGE_AUDIO {
    DataRange: KSDATARANGE {
        FormatSize: core::mem::size_of::<KSDATARANGE_AUDIO>() as u32,
        Flags: 0,
        SampleSize: 0,
        Reserved: 0,
        MajorFormat: KSDATAFORMAT_TYPE_AUDIO,
        SubFormat: KSDATAFORMAT_SUBTYPE_PCM,
        Specifier: KSDATAFORMAT_SPECIFIER_WAVEFORMATEX,
    },
    MaximumChannels: 2,
    MinimumBitsPerSample: 16,
    MaximumBitsPerSample: 32,
    MinimumSampleFrequency: 44100,
    MaximumSampleFrequency: 192000,
};

static FLOAT_DATARANGE: KSDATARANGE_AUDIO = KSDATARANGE_AUDIO {
    DataRange: KSDATARANGE {
        FormatSize: core::mem::size_of::<KSDATARANGE_AUDIO>() as u32,
        Flags: 0,
        SampleSize: 0,
        Reserved: 0,
        MajorFormat: KSDATAFORMAT_TYPE_AUDIO,
        SubFormat: KSDATAFORMAT_SUBTYPE_IEEE_FLOAT,
        Specifier: KSDATAFORMAT_SPECIFIER_WAVEFORMATEX,
    },
    MaximumChannels: 2,
    MinimumBitsPerSample: 32,
    MaximumBitsPerSample: 32,
    MinimumSampleFrequency: 44100,
    MaximumSampleFrequency: 192000,
};

#[allow(non_upper_case_globals)]
pub const KSCATEGORY_AUDIO_GUID: GUID = GUID {
    Data1: 0x69223398,
    Data2: 0x306C,
    Data3: 0x11CF,
    Data4: [0xB5, 0xCA, 0x00, 0x80, 0x5F, 0x48, 0xA1, 0x92],
};

#[allow(non_upper_case_globals)]
pub const KSCATEGORY_RENDER_GUID: GUID = GUID {
    Data1: 0x65E8773E,
    Data2: 0x8F56,
    Data3: 0x11D0,
    Data4: [0xA3, 0xB9, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub const KSCATEGORY_CAPTURE_GUID: GUID = GUID {
    Data1: 0x65E8773D,
    Data2: 0x8F56,
    Data3: 0x11D0,
    Data4: [0xA3, 0xB9, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub const KSNODETYPE_SPEAKER_GUID: GUID = GUID {
    Data1: 0xDFF219E1,
    Data2: 0xF70F,
    Data3: 0x11D0,
    Data4: [0xB9, 0x17, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

static BRIDGE_DATARANGE: KSDATARANGE = KSDATARANGE {
    FormatSize: core::mem::size_of::<KSDATARANGE>() as u32,
    Flags: 0,
    SampleSize: 0,
    Reserved: 0,
    MajorFormat: KSDATAFORMAT_TYPE_AUDIO,
    SubFormat: GUID {
        Data1: 0x00000000,
        Data2: 0x0000,
        Data3: 0x0000,
        Data4: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    }, // KSDATAFORMAT_SUBTYPE_NONE
    Specifier: GUID {
        Data1: 0x05589F81,
        Data2: 0xC356,
        Data3: 0x11CE,
        Data4: [0xBF, 0x01, 0x00, 0xAA, 0x00, 0x55, 0x59, 0x5A],
    }, // KSDATAFORMAT_SPECIFIER_NONE / WAVEFORMATEX
};

static BRIDGE_DATARANGES: [SyncPtr<KSDATARANGE>; 1] =
    [SyncPtr(&BRIDGE_DATARANGE as *const KSDATARANGE)];

static WAVE_RENDER_PINS: [PCPIN_DESCRIPTOR; 2] = [
    // Pin 0: Streaming Sink (Host -> Driver)
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: MAX_STREAMS as u32,
        MaxFilterInstanceCount: MAX_STREAMS as u32,
        MinFilterInstanceCount: 1,
        AutomationTable: core::ptr::null(),
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null(),
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 2,
            DataRanges: WAVE_DATARANGES.as_ptr() as *const *const KSDATARANGE,
            DataFlow: 1,      // KSPIN_DATAFLOW_IN
            Communication: 3, // KSPIN_COMMUNICATION_SINK
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: 0,
        },
    },
    // Pin 1: Bridge Source (Driver -> Topology)
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: core::ptr::null(),
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null(),
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *const KSDATARANGE,
            DataFlow: 2,      // KSPIN_DATAFLOW_OUT
            Communication: 1, // KSPIN_COMMUNICATION_BRIDGE
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: 0,
        },
    },
];

static WAVE_CAPTURE_PINS: [PCPIN_DESCRIPTOR; 2] = [
    // Pin 0: Bridge Sink (Topology -> Driver)
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: core::ptr::null(),
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null(),
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *const KSDATARANGE,
            DataFlow: 1,      // KSPIN_DATAFLOW_IN
            Communication: 1, // KSPIN_COMMUNICATION_BRIDGE
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: 0,
        },
    },
    // Pin 1: Streaming Source (Driver -> Host)
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: MAX_STREAMS as u32,
        MaxFilterInstanceCount: MAX_STREAMS as u32,
        MinFilterInstanceCount: 1,
        AutomationTable: core::ptr::null(),
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null(),
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 2,
            DataRanges: WAVE_DATARANGES.as_ptr() as *const *const KSDATARANGE,
            DataFlow: 2,      // KSPIN_DATAFLOW_OUT
            Communication: 2, // KSPIN_COMMUNICATION_SOURCE
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: 0,
        },
    },
];

static WAVE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromPin: KSPIN_WAVE_SINK,
    ToNode: PCFILTER_NODE,
    ToPin: KSPIN_WAVE_BRIDGE,
}];

static WAVE_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 1,
    AutomationTable: core::ptr::null(),
    PinSize: 2,
    PinDescriptorSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    Pins: WAVE_RENDER_PINS.as_ptr(),
    NodeSize: 0,
    NodeDescriptorSize: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: WAVE_CONNECTIONS.as_ptr() as *const u8,
    CategoryCount: 0,
    Categories: core::ptr::null(),
};

static WAVE_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 1,
    AutomationTable: core::ptr::null(),
    PinSize: 2,
    PinDescriptorSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    Pins: WAVE_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeDescriptorSize: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: WAVE_CONNECTIONS.as_ptr() as *const u8,
    CategoryCount: 0,
    Categories: core::ptr::null(),
};

// ============================================================================
// Topology Filter Description (Static)
// ============================================================================

static TOPO_RENDER_PINS: [PCPIN_DESCRIPTOR; 2] = [
    // Pin 0: Bridge Sink (Wave -> Topology)
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: core::ptr::null(),
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null(),
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *const KSDATARANGE,
            DataFlow: 1,      // KSPIN_DATAFLOW_IN
            Communication: 1, // KSPIN_COMMUNICATION_BRIDGE
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: 0,
        },
    },
    // Pin 1: Physical Source (The Endpoint)
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: core::ptr::null(),
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null(),
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *const KSDATARANGE,
            DataFlow: 2,      // KSPIN_DATAFLOW_OUT
            Communication: 0, // KSPIN_COMMUNICATION_NONE
            Category: &KSCATEGORY_RENDER_GUID,
            Name: core::ptr::null(),
            Reserved: 0,
        },
    },
];

static TOPO_CAPTURE_PINS: [PCPIN_DESCRIPTOR; 2] = [
    // Pin 0: Physical Sink (The External Source)
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: core::ptr::null(),
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null(),
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *const KSDATARANGE,
            DataFlow: 1,      // KSPIN_DATAFLOW_IN
            Communication: 0, // KSPIN_COMMUNICATION_NONE
            Category: &KSCATEGORY_CAPTURE_GUID,
            Name: core::ptr::null(),
            Reserved: 0,
        },
    },
    // Pin 1: Bridge Source (Topology -> Wave)
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: core::ptr::null(),
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null(),
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *const KSDATARANGE,
            DataFlow: 2,      // KSPIN_DATAFLOW_OUT
            Communication: 1, // KSPIN_COMMUNICATION_BRIDGE
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: 0,
        },
    },
];

static TOPO_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromPin: KSPIN_TOPO_BRIDGE,
    ToNode: PCFILTER_NODE,
    ToPin: KSPIN_TOPO_LINEOUT,
}];

static TOPO_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 1,
    AutomationTable: core::ptr::null(),
    PinSize: 2,
    PinDescriptorSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    Pins: TOPO_RENDER_PINS.as_ptr(),
    NodeSize: 0,
    NodeDescriptorSize: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: TOPO_CONNECTIONS.as_ptr() as *const u8,
    CategoryCount: 0,
    Categories: core::ptr::null(),
};

static TOPO_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 1,
    AutomationTable: core::ptr::null(),
    PinSize: 2,
    PinDescriptorSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    Pins: TOPO_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeDescriptorSize: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: TOPO_CONNECTIONS.as_ptr() as *const u8,
    CategoryCount: 0,
    Categories: core::ptr::null(),
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
    pub fn new(is_capture: bool) -> Box<Self> {
        Box::new(Self {
            vtable: &MINIPORT_VTABLE,
            inner: MiniportWaveRT::new(is_capture),
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
    this: *mut u8,
    out_description: *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTCom;
    let description = out_description as *mut *const PCFILTER_DESCRIPTOR;

    if (*com_obj).inner.is_capture {
        *description = &WAVE_CAPTURE_FILTER_DESCRIPTOR;
    } else {
        *description = &WAVE_RENDER_FILTER_DESCRIPTOR;
    }
    STATUS_SUCCESS
}

unsafe extern "system" fn miniport_data_range_intersection(
    _this: *mut u8,
    _pin_id: u32,
    data_range: *mut u8,
    _matching_data_range: *mut u8,
    data_format_cb: u32,
    data_format: *mut u8,
    actual_data_format_cb: *mut u32,
) -> NTSTATUS {
    if data_range.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let ks_range = data_range as *const KSDATARANGE;

    // 1. Verify Major Format is Audio.
    if !is_equal_guid(&(*ks_range).MajorFormat, &KSDATAFORMAT_TYPE_AUDIO) {
        return STATUS_NO_MATCH;
    }

    // 2. Verify Specifier is WaveFormatEx.
    if !is_equal_guid(&(*ks_range).Specifier, &KSDATAFORMAT_SPECIFIER_WAVEFORMATEX) {
        return STATUS_NO_MATCH;
    }

    // 3. Verify Subformat is PCM or Float.
    let is_pcm = is_equal_guid(&(*ks_range).SubFormat, &KSDATAFORMAT_SUBTYPE_PCM);
    let is_float = is_equal_guid(&(*ks_range).SubFormat, &KSDATAFORMAT_SUBTYPE_IEEE_FLOAT);

    if !is_pcm && !is_float {
        return STATUS_NO_MATCH;
    }

    // 4. Check for Buffer Size Query.
    let format_size = core::mem::size_of::<KSDATAFORMAT_WAVEFORMATEX>() as u32;

    if data_format_cb == 0 {
        if !actual_data_format_cb.is_null() {
            *actual_data_format_cb = format_size;
        }
        return STATUS_BUFFER_OVERFLOW;
    }

    if data_format_cb < format_size {
        return STATUS_BUFFER_TOO_SMALL;
    }

    // 5. Fill Resultant Format.
    let result = data_format as *mut KSDATAFORMAT_WAVEFORMATEX;

    (*result).DataFormat.FormatSize = format_size;
    (*result).DataFormat.Flags = 0;
    (*result).DataFormat.SampleSize = 0;
    (*result).DataFormat.Reserved = 0;
    (*result).DataFormat.MajorFormat = KSDATAFORMAT_TYPE_AUDIO;
    (*result).DataFormat.SubFormat = (*ks_range).SubFormat;
    (*result).DataFormat.Specifier = KSDATAFORMAT_SPECIFIER_WAVEFORMATEX;

    (*result).WaveFormatEx.wFormatTag = if is_pcm { 1 } else { 3 }; // WAVE_FORMAT_PCM=1, WAVE_FORMAT_IEEE_FLOAT=3
    (*result).WaveFormatEx.nChannels = 2;
    (*result).WaveFormatEx.nSamplesPerSec = 48000;
    (*result).WaveFormatEx.wBitsPerSample = if is_pcm { 16 } else { 32 };
    (*result).WaveFormatEx.nBlockAlign =
        ((*result).WaveFormatEx.nChannels * (*result).WaveFormatEx.wBitsPerSample) / 8;
    (*result).WaveFormatEx.nAvgBytesPerSec =
        (*result).WaveFormatEx.nSamplesPerSec * (*result).WaveFormatEx.nBlockAlign as u32;
    (*result).WaveFormatEx.cbSize = 0;

    if !actual_data_format_cb.is_null() {
        *actual_data_format_cb = format_size;
    }

    STATUS_SUCCESS
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
    pub is_capture: bool,
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
    pub fn new(is_capture: bool) -> Self {
        Self {
            max_pci_bar: 0,
            is_initialized: false,
            is_capture,
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

    // --- LEYLINE OUTPUT (RENDER) ---

    // 1. Create the WaveRT Port object.
    let mut render_port: *mut u8 = core::ptr::null_mut();
    status = PcNewPort(&mut render_port, &CLSID_PortWaveRT);
    if status != STATUS_SUCCESS {
        return status;
    }

    // 2. Create the Miniport object.
    let render_miniport_com = MiniportWaveRTCom::new(false); // false = render
    let render_miniport_ptr = Box::into_raw(render_miniport_com) as *mut u8;

    // 3. Initialize the Port with the Miniport.
    type PortInitFn = unsafe extern "system" fn(
        this: *mut u8,
        device_object: PDEVICE_OBJECT,
        irp: PIRP,
        miniport: *mut u8,
        unknown_adapter: PVOID,
        resource_list: PVOID,
    ) -> NTSTATUS;

    let vtable = *(render_port as *const *const *const u8);
    let init_ptr = *vtable.add(3);
    let init_fn: PortInitFn = core::mem::transmute(init_ptr);

    status = init_fn(
        render_port,
        device_object,
        core::ptr::null_mut(),
        render_miniport_ptr,
        core::ptr::null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        return status;
    }

    // 4. Register the WaveRT subdevice.
    let wave_render_name: [u16; 11] = [
        0x0057, 0x0061, 0x0076, 0x0065, 0x0052, 0x0065, 0x006E, 0x0064, 0x0065, 0x0072, 0x0000,
    ]; // "WaveRender\0"
    status = PcRegisterSubdevice(device_object, wave_render_name.as_ptr(), render_port);
    if status != STATUS_SUCCESS {
        return status;
    }

    // 5. Create the Topology Port object.
    let mut render_topo_port: *mut u8 = core::ptr::null_mut();
    status = PcNewPort(&mut render_topo_port, &CLSID_PortTopology);
    if status != STATUS_SUCCESS {
        return status;
    }

    // 6. Create the Topology Miniport object.
    let render_topo_miniport_com = MiniportTopologyCom::new(false);
    let render_topo_miniport_ptr = Box::into_raw(render_topo_miniport_com) as *mut u8;

    // 7. Initialize the Topology Port with the Miniport.
    status = init_fn(
        render_topo_port,
        device_object,
        core::ptr::null_mut(),
        render_topo_miniport_ptr,
        core::ptr::null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        return status;
    }

    // 8. Register the Topology subdevice.
    let topo_render_name: [u16; 11] = [
        0x0054, 0x006F, 0x0070, 0x006F, 0x0052, 0x0065, 0x006E, 0x0064, 0x0065, 0x0072, 0x0000,
    ]; // "TopoRender\0"
    status = PcRegisterSubdevice(device_object, topo_render_name.as_ptr(), render_topo_port);
    if status != STATUS_SUCCESS {
        return status;
    }

    // 9. Link the Wave filter to the Topology filter.
    status = PcRegisterPhysicalConnection(
        device_object,
        render_port,
        KSPIN_WAVE_BRIDGE,
        render_topo_port,
        KSPIN_TOPO_BRIDGE,
    );
    if status != STATUS_SUCCESS {
        return status;
    }

    // --- LEYLINE INPUT (CAPTURE) ---

    // 10. Create the WaveRT Port object for Capture.
    let mut capture_port: *mut u8 = core::ptr::null_mut();
    status = PcNewPort(&mut capture_port, &CLSID_PortWaveRT);
    if status != STATUS_SUCCESS {
        return status;
    }

    // 11. Create the Miniport object for Capture.
    let capture_miniport_com = MiniportWaveRTCom::new(true); // true = capture
    let capture_miniport_ptr = Box::into_raw(capture_miniport_com) as *mut u8;

    // 12. Initialize the Capture Port.
    status = init_fn(
        capture_port,
        device_object,
        core::ptr::null_mut(),
        capture_miniport_ptr,
        core::ptr::null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        return status;
    }

    // 13. Register the Capture WaveRT subdevice.
    let wave_capture_name: [u16; 12] = [
        0x0057, 0x0061, 0x0076, 0x0065, 0x0043, 0x0061, 0x0070, 0x0074, 0x0075, 0x0072, 0x0065,
        0x0000,
    ]; // "WaveCapture\0"
    status = PcRegisterSubdevice(device_object, wave_capture_name.as_ptr(), capture_port);
    if status != STATUS_SUCCESS {
        return status;
    }

    // 14. Create the Topology Port object for Capture.
    let mut capture_topo_port: *mut u8 = core::ptr::null_mut();
    status = PcNewPort(&mut capture_topo_port, &CLSID_PortTopology);
    if status != STATUS_SUCCESS {
        return status;
    }

    // 15. Create the Topology Miniport object for Capture.
    let capture_topo_miniport_com = MiniportTopologyCom::new(true);
    let capture_topo_miniport_ptr = Box::into_raw(capture_topo_miniport_com) as *mut u8;

    // 16. Initialize the Capture Topology Port.
    status = init_fn(
        capture_topo_port,
        device_object,
        core::ptr::null_mut(),
        capture_topo_miniport_ptr,
        core::ptr::null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        return status;
    }

    // 17. Register the Capture Topology subdevice.
    let topo_capture_name: [u16; 12] = [
        0x0054, 0x006F, 0x0070, 0x006F, 0x0043, 0x0061, 0x0070, 0x0074, 0x0075, 0x0072, 0x0065,
        0x0000,
    ]; // "TopoCapture\0"
    status = PcRegisterSubdevice(device_object, topo_capture_name.as_ptr(), capture_topo_port);
    if status != STATUS_SUCCESS {
        return status;
    }

    // 18. Link the Capture Wave filter to the Capture Topology filter.
    // Note: For capture, flow is Topo -> Wave
    status = PcRegisterPhysicalConnection(
        device_object,
        capture_topo_port,
        KSPIN_TOPO_LINEOUT, // For capture, this is the "Source" bridge
        capture_port,
        KSPIN_TOPO_BRIDGE, // This constant ID happens to be 0
    );

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
