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
use wdk_sys::ntddk::*;
use wdk_sys::*;

// ============================================================================
// PortCls External Declarations
// ============================================================================

extern "C" {
    pub fn DbgPrint(Format: *const u8, ...) -> u32;
}

// ============================================================================
// PortCls External Declarations
// ============================================================================

#[link(name = "portcls")]
extern "C" {
    pub fn PcAddAdapterDevice(
        DriverObject: PDRIVER_OBJECT,
        PhysicalDeviceObject: PDEVICE_OBJECT,
        StartDevice: Option<unsafe extern "C" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS>,
        MaxObjects: u32,
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

#[allow(non_upper_case_globals)]
pub const CLSID_PortWaveRT: GUID = GUID {
    Data1: 0xCC9BE57A,
    Data2: 0xEB9E,
    Data3: 0x42B4,
    Data4: [0x94, 0xFC, 0x0C, 0xAD, 0x3D, 0xBC, 0xE7, 0xFA],
};

#[allow(non_upper_case_globals)]
pub const IID_IPortWaveRT: GUID = GUID {
    Data1: 0x339FF909,
    Data2: 0x68A9,
    Data3: 0x4310,
    Data4: [0xB0, 0x9B, 0x27, 0x4E, 0x96, 0xEE, 0x4C, 0xBD],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportWaveRT: GUID = GUID {
    Data1: 0x0F9FC4D6,
    Data2: 0x6061,
    Data3: 0x4F3C,
    Data4: [0xB1, 0xFC, 0x07, 0x5E, 0x35, 0xF7, 0x96, 0x0A],
};

#[allow(non_upper_case_globals)]
pub const IID_IPortWaveRTStream: GUID = GUID {
    Data1: 0x1809CE5A,
    Data2: 0x64BC,
    Data3: 0x4E62,
    Data4: [0xBD, 0x7D, 0x95, 0xBC, 0xE4, 0x3D, 0xE3, 0x93],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportWaveRTStream: GUID = GUID {
    Data1: 0x000AC9AB,
    Data2: 0xFAAB,
    Data3: 0x4F3D,
    Data4: [0x94, 0x55, 0x6F, 0xF8, 0x30, 0x6A, 0x74, 0xA0],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportWaveRTStreamNotification: GUID = GUID {
    Data1: 0x23759128,
    Data2: 0x96F1,
    Data3: 0x423B,
    Data4: [0xAB, 0x4D, 0x81, 0x63, 0x5B, 0xCF, 0x8C, 0xA1],
};

#[allow(non_upper_case_globals)]
pub const CLSID_PortTopology: GUID = GUID {
    Data1: 0xB4C90A32,
    Data2: 0x5791,
    Data3: 0x11D0,
    Data4: [0x86, 0xF9, 0x00, 0xA0, 0xC9, 0x11, 0xB5, 0x44],
};

#[allow(non_upper_case_globals)]
pub const IID_IPortTopology: GUID = GUID {
    Data1: 0xB4C90A30,
    Data2: 0x5791,
    Data3: 0x11D0,
    Data4: [0x86, 0xF9, 0x00, 0xA0, 0xC9, 0x11, 0xB5, 0x44],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportTopology: GUID = GUID {
    Data1: 0xB4C90A31,
    Data2: 0x5791,
    Data3: 0x11D0,
    Data4: [0x86, 0xF9, 0x00, 0xA0, 0xC9, 0x11, 0xB5, 0x44],
};

#[allow(non_upper_case_globals)]
pub const IID_IPort: GUID = GUID {
    Data1: 0xB4C90A25,
    Data2: 0x5791,
    Data3: 0x11D0,
    Data4: [0x86, 0xF9, 0x00, 0xA0, 0xC9, 0x11, 0xB5, 0x44],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniport: GUID = GUID {
    Data1: 0xB4C90A24,
    Data2: 0x5791,
    Data3: 0x11D0,
    Data4: [0x86, 0xF9, 0x00, 0xA0, 0xC9, 0x11, 0xB5, 0x44],
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
pub const KSDATAFORMAT_SUBTYPE_ANALOG: GUID = GUID {
    Data1: 0x6DBA3190,
    Data2: 0x67BD,
    Data3: 0x11CF,
    Data4: [0xA0, 0xF7, 0x00, 0x20, 0xAF, 0xD1, 0x56, 0xE4],
};

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_SPECIFIER_WAVEFORMATEX: GUID = GUID {
    Data1: 0x05589F81,
    Data2: 0xC356,
    Data3: 0x11CE,
    Data4: [0xBF, 0x01, 0x00, 0xAA, 0x00, 0x55, 0x59, 0x5A],
};

#[allow(non_upper_case_globals)]
pub const KSCATEGORY_AUDIO_GUID: GUID = GUID {
    Data1: 0x6994AD04,
    Data2: 0x93EF,
    Data3: 0x11D0,
    Data4: [0xA3, 0xCC, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
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
pub const KSCATEGORY_TOPOLOGY_GUID: GUID = GUID {
    Data1: 0xDDA54A40,
    Data2: 0x1E4C,
    Data3: 0x11D1,
    Data4: [0xA0, 0x50, 0x40, 0x57, 0x05, 0xC1, 0x00, 0x00],
};

#[allow(non_upper_case_globals)]
pub const KSCATEGORY_REALTIME_GUID: GUID = GUID {
    Data1: 0xEB115AD5,
    Data2: 0x9118,
    Data3: 0x4FA0,
    Data4: [0xBD, 0x83, 0xED, 0x35, 0x22, 0x15, 0xDF, 0x43],
};

#[allow(non_upper_case_globals)]
pub const KSNODETYPE_SPEAKER: GUID = GUID {
    Data1: 0xDFF21CE1,
    Data2: 0xF30F,
    Data3: 0x11D0,
    Data4: [0xA9, 0x71, 0x00, 0xAA, 0x00, 0x61, 0x52, 0x93],
};

#[allow(non_upper_case_globals)]
pub const KSNODETYPE_MICROPHONE: GUID = GUID {
    Data1: 0xDFF21BE1,
    Data2: 0xF30F,
    Data3: 0x11D0,
    Data4: [0xA9, 0x71, 0x00, 0xAA, 0x00, 0x61, 0x52, 0x93],
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
    pub Init: unsafe extern "system" fn(
        this: *mut u8,
        unknown_adapter: *mut u8,
        resource_list: *mut u8,
        port: *mut u8,
    ) -> NTSTATUS,
    pub NewStream: unsafe extern "system" fn(
        this: *mut u8,
        stream: *mut *mut u8,
        port_stream: *mut u8,
        pin: u32,
        capture: bool,
        format: *mut u8,
    ) -> NTSTATUS,
    pub GetDeviceDescription:
        unsafe extern "system" fn(this: *mut u8, description: *mut u8) -> NTSTATUS,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct IMiniportWaveRTStreamVTable {
    pub base: IUnknownVTable,
    pub SetFormat: unsafe extern "system" fn(this: *mut u8, format: *mut u8) -> NTSTATUS,
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
    pub GetPositionRegister:
        unsafe extern "system" fn(this: *mut u8, position_register: *mut u8) -> NTSTATUS,
    pub GetClockRegister:
        unsafe extern "system" fn(this: *mut u8, clock_register: *mut u8) -> NTSTATUS,
}

// ============================================================================
// Constants & Configuration
// ============================================================================

const _POOL_TAG: u32 = u32::from_be_bytes(*b"LLAD");
const MAX_STREAMS: usize = 4;

/// PortCls reserves the first part of the device extension.
/// On x64, this is 512 bytes (64 * sizeof(ULONG_PTR)).
const PORT_CLASS_DEVICE_EXTENSION_SIZE: usize = 64 * core::mem::size_of::<usize>();

#[inline(always)]
unsafe fn get_device_extension(device_object: PDEVICE_OBJECT) -> *mut DeviceExtension {
    let base = (*device_object).DeviceExtension as *mut u8;
    base.add(PORT_CLASS_DEVICE_EXTENSION_SIZE) as *mut DeviceExtension
}

#[global_allocator]
static GLOBAL: WDKAllocator = WDKAllocator;

// ============================================================================
// Device Extension
// ============================================================================

#[repr(C)]
pub struct DeviceExtension {
    pub control_device_object: *mut DEVICE_OBJECT,
    pub shared_params: *mut leyline_shared::SharedParameters,
    pub shared_params_mdl: PMDL,
    pub shared_params_user_mapping: *mut u8,
    pub loopback_mdl: PMDL,
    pub loopback_buffer: *mut u8,
    pub loopback_size: usize,
    pub user_mapping: *mut u8, // Tracks user-mode mapping for cleanup
    pub render_miniport: *mut MiniportWaveRTCom,
    pub capture_miniport: *mut MiniportWaveRTCom,
    pub render_topo_miniport: *mut MiniportTopologyCom,
    pub capture_topo_miniport: *mut MiniportTopologyCom,
}

// ============================================================================
// Miniport Structure
// ============================================================================

pub struct MiniportWaveRT {
    pub max_pci_bar: u32,
    pub is_initialized: bool,
    pub is_capture: bool,
    pub streams: [Option<Box<MiniportWaveRTStream>>; MAX_STREAMS],
    pub device_extension: *mut DeviceExtension,
}

impl MiniportWaveRT {
    pub fn new(is_capture: bool, device_extension: *mut DeviceExtension) -> Self {
        Self {
            max_pci_bar: 0,
            is_initialized: false,
            is_capture,
            streams: [None, None, None, None],
            device_extension,
        }
    }

    pub fn init(&mut self, _ua: PVOID, _rl: PVOID, _p: PVOID) -> NTSTATUS {
        self.is_initialized = true;
        STATUS_SUCCESS
    }

    pub fn new_stream(&mut self, _pin: u32, _cap: bool, _fmt: PVOID) -> *mut MiniportWaveRTStream {
        if !self.is_initialized {
            return core::ptr::null_mut();
        }
        for slot in self.streams.iter_mut() {
            if slot.is_none() {
                unsafe {
                    *slot = Some(Box::new(MiniportWaveRTStream::new(
                        _fmt,
                        _cap,
                        Box::new(crate::stream::KernelTimeSource) as Box<dyn TimeSource>,
                        self.device_extension as *mut u8,
                    )));
                }
                return slot.as_mut().unwrap().as_mut() as *mut MiniportWaveRTStream;
            }
        }
        core::ptr::null_mut()
    }

    pub fn get_device_description(&self, _dd: PDEVICE_DESCRIPTION) -> NTSTATUS {
        STATUS_SUCCESS
    }
}

#[repr(C)]
pub struct MiniportWaveRTCom {
    pub vtable: *const IMiniportWaveRTVTable,
    pub inner: MiniportWaveRT,
    pub ref_count: u32,
}

impl MiniportWaveRTCom {
    pub fn new(is_capture: bool, device_extension: *mut DeviceExtension) -> Box<Self> {
        Box::new(Self {
            vtable: &MINIPORT_VTABLE,
            inner: MiniportWaveRT::new(is_capture, device_extension),
            ref_count: 1,
        })
    }
}

// ============================================================================
// Miniport VTable Callbacks
// ============================================================================

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
    _pin: u32,
    data_range: *mut u8,
    _matching: *mut u8,
    data_format_cb: u32,
    data_format: *mut u8,
    actual_data_format_cb: *mut u32,
) -> NTSTATUS {
    if data_range.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let ks_range = data_range as *const KSDATARANGE;
    if !is_equal_guid(&(*ks_range).MajorFormat, &KSDATAFORMAT_TYPE_AUDIO) {
        return STATUS_NO_MATCH;
    }
    if !is_equal_guid(&(*ks_range).Specifier, &KSDATAFORMAT_SPECIFIER_WAVEFORMATEX) {
        return STATUS_NO_MATCH;
    }
    let is_pcm = is_equal_guid(&(*ks_range).SubFormat, &KSDATAFORMAT_SUBTYPE_PCM);
    let is_float = is_equal_guid(&(*ks_range).SubFormat, &KSDATAFORMAT_SUBTYPE_IEEE_FLOAT);
    if !is_pcm && !is_float {
        return STATUS_NO_MATCH;
    }

    let format_size = core::mem::size_of::<KSDATAFORMAT_WAVEFORMATEX>() as u32;
    if data_format_cb == 0 {
        if !actual_data_format_cb.is_null() {
            unsafe {
                *actual_data_format_cb = format_size;
            }
        }
        return STATUS_BUFFER_OVERFLOW;
    }
    if data_format_cb < format_size {
        return STATUS_BUFFER_TOO_SMALL;
    }

    let result = data_format as *mut KSDATAFORMAT_WAVEFORMATEX;
    unsafe {
        (*result).DataFormat.FormatSize = format_size;
        (*result).DataFormat.MajorFormat = KSDATAFORMAT_TYPE_AUDIO;
        (*result).DataFormat.SubFormat = (*ks_range).SubFormat;
        (*result).DataFormat.Specifier = KSDATAFORMAT_SPECIFIER_WAVEFORMATEX;

        (*result).WaveFormatEx.wFormatTag = if is_pcm { 1 } else { 3 };
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
    _port: *mut u8,
    pin: u32,
    capture: bool,
    format: *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTCom;
    let stream_ptr = (*com_obj).inner.new_stream(pin, capture, format as PVOID);
    if stream_ptr.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
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

fn is_equal_guid(a: *const GUID, b: &GUID) -> bool {
    unsafe {
        (*a).Data1 == b.Data1
            && (*a).Data2 == b.Data2
            && (*a).Data3 == b.Data3
            && (*a).Data4 == b.Data4
    }
}

// ============================================================================
// Miniport Topology
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
    pub fn init(&mut self, _ua: PVOID, _rl: PVOID, _p: PVOID) -> NTSTATUS {
        self.is_initialized = true;
        STATUS_SUCCESS
    }
}

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
    _pin: u32,
    _dr: *mut u8,
    _mdr: *mut u8,
    _dfcb: u32,
    _df: *mut u8,
    _adfcb: *mut u32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

unsafe extern "system" fn topology_init(
    this: *mut u8,
    ua: *mut u8,
    rl: *mut u8,
    p: *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportTopologyCom;
    (*com_obj).inner.init(ua as PVOID, rl as PVOID, p as PVOID)
}

static TOPOLOGY_VTABLE: IMiniportTopologyVTable = IMiniportTopologyVTable {
    base: IUnknownVTable {
        QueryInterface: topology_query_interface,
        AddRef: topology_add_ref,
        Release: topology_release,
    },
    Init: topology_init,
    GetDescription: topology_get_description,
    DataRangeIntersection: topology_data_range_intersection,
};

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
    if is_equal_guid(iid, &IID_IMiniportWaveRTStream) || is_equal_guid(iid, &IID_IUnknown) {
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

unsafe extern "system" fn stream_set_format(_this: *mut u8, _format: *mut u8) -> NTSTATUS {
    STATUS_SUCCESS
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
    req_size: usize,
    mdl: *mut *mut u8,
    act_size: *mut usize,
    off: *mut u32,
    cache: *mut i32,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    let status = (*(*com_obj).stream).allocate_audio_buffer(req_size, mdl as *mut PMDL);
    if status == STATUS_SUCCESS {
        if !act_size.is_null() {
            *act_size = req_size;
        }
        if !off.is_null() {
            *off = 0;
        }
        if !cache.is_null() {
            *cache = 1;
        }
    }
    status
}

unsafe extern "system" fn stream_free_audio_buffer(_this: *mut u8, _mdl: *mut u8, _size: usize) {}
unsafe extern "system" fn stream_get_hw_latency(this: *mut u8, latency: *mut u32) {
    let com_obj = this as *mut MiniportWaveRTStreamCom;
    (*(*com_obj).stream).get_hw_latency(latency);
}
unsafe extern "system" fn stream_get_position_register(_this: *mut u8, _reg: *mut u8) -> NTSTATUS {
    0xC00000BBu32 as i32
}
unsafe extern "system" fn stream_get_clock_register(_this: *mut u8, _reg: *mut u8) -> NTSTATUS {
    0xC00000BBu32 as i32
}

static STREAM_VTABLE: IMiniportWaveRTStreamVTable = IMiniportWaveRTStreamVTable {
    base: IUnknownVTable {
        QueryInterface: stream_query_interface,
        AddRef: stream_add_ref,
        Release: stream_release,
    },
    SetFormat: stream_set_format,
    SetState: stream_set_state,
    GetPosition: stream_get_position,
    AllocateAudioBuffer: stream_allocate_audio_buffer,
    FreeAudioBuffer: stream_free_audio_buffer,
    GetHWLatency: stream_get_hw_latency,
    GetPositionRegister: stream_get_position_register,
    GetClockRegister: stream_get_clock_register,
};

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
const KSPIN_WAVE_SINK: u32 = 0;
const KSPIN_WAVE_BRIDGE: u32 = 1;
const KSPIN_TOPO_BRIDGE: u32 = 0;
const KSPIN_TOPO_LINEOUT: u32 = 1;

#[repr(transparent)]
struct SyncPtr<T>(*const T);
unsafe impl<T> Sync for SyncPtr<T> {}

#[allow(dead_code)]
static WAVE_DATARANGES: [SyncPtr<KSDATARANGE>; 2] = [
    SyncPtr(&PCM_DATARANGE.DataRange as *const KSDATARANGE),
    SyncPtr(&FLOAT_DATARANGE.DataRange as *const KSDATARANGE),
];

#[allow(dead_code)]
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

#[allow(dead_code)]
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

static BRIDGE_DATARANGE: KSDATARANGE = KSDATARANGE {
    FormatSize: core::mem::size_of::<KSDATARANGE>() as u32,
    Flags: 0,
    SampleSize: 0,
    Reserved: 0,
    MajorFormat: KSDATAFORMAT_TYPE_AUDIO,
    SubFormat: KSDATAFORMAT_SUBTYPE_ANALOG,
    Specifier: KSDATAFORMAT_SPECIFIER_WAVEFORMATEX,
};

static BRIDGE_DATARANGES: [SyncPtr<KSDATARANGE>; 1] =
    [SyncPtr(&BRIDGE_DATARANGE as *const KSDATARANGE)];

#[allow(dead_code)]
static WAVE_RENDER_PINS: [PCPIN_DESCRIPTOR; 2] = [
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
            Communication: 3, // KSPIN_COMMUNICATION_BOTH
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
        },
    },
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
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
        },
    },
];

#[allow(dead_code)]
static WAVE_CAPTURE_PINS: [PCPIN_DESCRIPTOR; 2] = [
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
            Communication: 3, // KSPIN_COMMUNICATION_BOTH
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
        },
    },
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
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
        },
    },
];

#[allow(dead_code)]
static WAVE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromPin: KSPIN_WAVE_SINK,
    ToNode: PCFILTER_NODE,
    ToPin: KSPIN_WAVE_BRIDGE,
}];

#[allow(dead_code)]
static WAVE_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: core::ptr::null(),
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: WAVE_RENDER_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: WAVE_CONNECTIONS.as_ptr() as *const u8,
    CategoryCount: 0,
    Categories: core::ptr::null(),
};

#[allow(dead_code)]
static WAVE_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: core::ptr::null(),
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: WAVE_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: WAVE_CONNECTIONS.as_ptr() as *const u8,
    CategoryCount: 0,
    Categories: core::ptr::null(),
};

static TOPO_RENDER_PINS: [PCPIN_DESCRIPTOR; 2] = [
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
            DataFlow: 1,
            Communication: 1,
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
        },
    },
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
            DataFlow: 2,
            Communication: 0,
            Category: &KSNODETYPE_SPEAKER,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
        },
    },
];

static TOPO_CAPTURE_PINS: [PCPIN_DESCRIPTOR; 2] = [
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
            DataFlow: 1,
            Communication: 0,
            Category: &KSNODETYPE_MICROPHONE,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
        },
    },
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
            DataFlow: 2,
            Communication: 1,
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
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
    Version: 0,
    AutomationTable: core::ptr::null(),
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: TOPO_RENDER_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: TOPO_CONNECTIONS.as_ptr() as *const u8,
    CategoryCount: 0,
    Categories: core::ptr::null(),
};

static TOPO_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: core::ptr::null(),
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: TOPO_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: TOPO_CONNECTIONS.as_ptr() as *const u8,
    CategoryCount: 0,
    Categories: core::ptr::null(),
};

// ============================================================================
// Global Driver State
// ============================================================================

static mut CONTROL_DEVICE_OBJECT: *mut DEVICE_OBJECT = core::ptr::null_mut();

static mut ETW_REG_HANDLE: u64 = 0;

/// Leyline Audio Driver ETW Provider GUID: {71549463-5E1E-4B7E-9F93-A65606E50D64}
const ETW_PROVIDER_GUID: GUID = GUID {
    Data1: 0x71549463,
    Data2: 0x5E1E,
    Data3: 0x4B7E,
    Data4: [0x9F, 0x93, 0xA6, 0x56, 0x06, 0xE5, 0x0D, 0x64],
};

// ============================================================================
// Driver Entry Point
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn DriverEntry(
    driver_object: PDRIVER_OBJECT,
    registry_path: PUNICODE_STRING,
) -> NTSTATUS {
    DbgPrint("Leyline: DriverEntry\n\0".as_ptr());

    // Register ETW Provider for professional telemetry
    let status = EtwRegister(
        &ETW_PROVIDER_GUID,
        None,
        core::ptr::null_mut(),
        &raw mut ETW_REG_HANDLE,
    );
    if status == 0 {
        DbgPrint("Leyline: ETW Provider Registered Successfully\n\0".as_ptr());
    }

    (*driver_object).DriverUnload = Some(DriverUnload);

    let status = PcInitializeAdapterDriver(driver_object, registry_path, Some(AddDevice));
    if status == STATUS_SUCCESS {
        DbgPrint("Leyline: PcInitializeAdapterDriver Success\n\0".as_ptr());
    } else {
        DbgPrint(
            "Leyline: PcInitializeAdapterDriver Failed with status 0x%08X\n\0".as_ptr(),
            status,
        );
    }
    status
}

#[allow(non_snake_case)]
pub unsafe extern "C" fn DriverUnload(_driver_object: PDRIVER_OBJECT) {
    DbgPrint("Leyline: DriverUnload\n\0".as_ptr());
    if ETW_REG_HANDLE != 0 {
        let _ = EtwUnregister(ETW_REG_HANDLE);
        ETW_REG_HANDLE = 0;
        DbgPrint("Leyline: ETW Provider Unregistered\n\0".as_ptr());
    }
}

#[no_mangle]
pub unsafe extern "C" fn AddDevice(
    driver_object: PDRIVER_OBJECT,
    physical_device_object: PDEVICE_OBJECT,
) -> NTSTATUS {
    DbgPrint("Leyline: AddDevice\n\0".as_ptr());

    if driver_object.is_null() || physical_device_object.is_null() {
        DbgPrint("Leyline: AddDevice received NULL parameters\n\0".as_ptr());
        return STATUS_INVALID_PARAMETER;
    }

    // PortCls requires the total size of the device extension, including its own
    // PORT_CLASS_DEVICE_EXTENSION structure at the beginning.
    let total_extension_size =
        (PORT_CLASS_DEVICE_EXTENSION_SIZE + core::mem::size_of::<DeviceExtension>()) as u32;

    DbgPrint(
        "Leyline: Calling PcAddAdapterDevice with DriverObject=%p, PDO=%p, TotalExtSize=%u\n\0"
            .as_ptr(),
        driver_object,
        physical_device_object,
        total_extension_size,
    );

    let status = PcAddAdapterDevice(
        driver_object,
        physical_device_object,
        Some(StartDevice),
        10, // MaxSubDevices (Common value)
        total_extension_size,
    );

    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: PcAddAdapterDevice failed with status 0x%08X\n\0".as_ptr(),
            status,
        );
    }
    status
}
#[no_mangle]
pub unsafe extern "C" fn StartDevice(
    device_object: PDEVICE_OBJECT,
    _irp: PIRP,
    resource_list: PVOID,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let dev_ext = get_device_extension(device_object);
    DbgPrint("Leyline: StartDevice\n\0".as_ptr());

    if (*dev_ext).shared_params.is_null() {
        DbgPrint("Leyline: Allocating Shared Memory\n\0".as_ptr());
        let params = ExAllocatePool2(
            POOL_FLAG_NON_PAGED,
            core::mem::size_of::<leyline_shared::SharedParameters>() as u64,
            _POOL_TAG,
        );
        if params.is_null() {
            DbgPrint("Leyline: Shared memory allocation FAILED\n\0".as_ptr());
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        (*dev_ext).shared_params = params as *mut leyline_shared::SharedParameters;
        (*(*dev_ext).shared_params).master_gain_bits = 0x3F800000;

        (*dev_ext).shared_params_mdl = IoAllocateMdl(
            params,
            core::mem::size_of::<leyline_shared::SharedParameters>() as u32,
            0,
            0,
            core::ptr::null_mut(),
        );
        if !(*dev_ext).shared_params_mdl.is_null() {
            MmBuildMdlForNonPagedPool((*dev_ext).shared_params_mdl);
        }
        (*dev_ext).shared_params_user_mapping = core::ptr::null_mut();

        let buffer_size = 64 * 1024;
        let low: PHYSICAL_ADDRESS = core::mem::zeroed();
        let mut high: PHYSICAL_ADDRESS = core::mem::zeroed();
        high.QuadPart = 0xFFFFFFFF;
        let skip: PHYSICAL_ADDRESS = core::mem::zeroed();

        (*dev_ext).loopback_mdl = MmAllocatePagesForMdlEx(
            low,
            high,
            skip,
            buffer_size as u64,
            _MEMORY_CACHING_TYPE::MmCached,
            MM_ALLOCATE_FULLY_REQUIRED,
        );

        if !(*dev_ext).loopback_mdl.is_null() {
            (*dev_ext).loopback_buffer = MmMapLockedPagesSpecifyCache(
                (*dev_ext).loopback_mdl,
                0,
                _MEMORY_CACHING_TYPE::MmCached,
                core::ptr::null_mut(),
                0,
                _MM_PAGE_PRIORITY::NormalPagePriority as u32,
            ) as *mut u8;
            (*dev_ext).loopback_size = buffer_size;
            (*dev_ext).user_mapping = core::ptr::null_mut();
            DbgPrint("Leyline: Loopback Buffer Ready\n\0".as_ptr());
        } else {
            DbgPrint("Leyline: MDL allocation FAILED\n\0".as_ptr());
        }
    }

    if CONTROL_DEVICE_OBJECT.is_null() {
        let mut device_name_str = [0u16; 20];
        let name_prefix = r"\Device\LeylineAudio";
        for (i, c) in name_prefix.encode_utf16().enumerate() {
            device_name_str[i] = c;
        }
        let mut device_name = UNICODE_STRING {
            Length: (name_prefix.len() * 2) as u16,
            MaximumLength: (device_name_str.len() * 2) as u16,
            Buffer: device_name_str.as_mut_ptr(),
        };
        DbgPrint("Leyline: Creating CDO\n\0".as_ptr());
        status = IoCreateDevice(
            (*device_object).DriverObject,
            core::mem::size_of::<usize>() as u32,
            &mut device_name,
            FILE_DEVICE_UNKNOWN,
            0,
            0,
            &raw mut CONTROL_DEVICE_OBJECT,
        );
        if status == STATUS_SUCCESS {
            let cdo_ext = (*CONTROL_DEVICE_OBJECT).DeviceExtension as *mut usize;
            *cdo_ext = dev_ext as usize;

            let mut link_name_str = [0u16; 25];
            let link_prefix = r"\DosDevices\LeylineAudio";
            for (i, c) in link_prefix.encode_utf16().enumerate() {
                link_name_str[i] = c;
            }
            let mut link_name = UNICODE_STRING {
                Length: (link_prefix.len() * 2) as u16,
                MaximumLength: (link_name_str.len() * 2) as u16,
                Buffer: link_name_str.as_mut_ptr(),
            };
            status = IoCreateSymbolicLink(&mut link_name, &mut device_name);
            if status != STATUS_SUCCESS {
                DbgPrint(
                    "Leyline: IoCreateSymbolicLink FAILED (0x%08X)\n\0".as_ptr(),
                    status,
                );
                IoDeleteDevice(CONTROL_DEVICE_OBJECT);
                CONTROL_DEVICE_OBJECT = core::ptr::null_mut();
                return status;
            }
            DbgPrint("Leyline: CDO Ready\n\0".as_ptr());
        } else {
            DbgPrint(
                "Leyline: IoCreateDevice FAILED (0x%08X)\n\0".as_ptr(),
                status,
            );
            return status;
        }
    }

    DbgPrint("Leyline: Registering WaveRender Port\n\0".as_ptr());
    let mut render_port: *mut u8 = core::ptr::null_mut();
    status = PcNewPort(&mut render_port, &CLSID_PortWaveRT);
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: PcNewPort WaveRT FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }
    let render_miniport_com = MiniportWaveRTCom::new(false, dev_ext);
    let render_miniport_ptr = Box::into_raw(render_miniport_com) as *mut u8;
    (*dev_ext).render_miniport = render_miniport_ptr as *mut MiniportWaveRTCom;

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
        _irp,
        render_miniport_ptr,
        core::ptr::null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: Port Init WaveRT FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }

    let wave_render_name: [u16; 11] = [
        0x0057, 0x0061, 0x0076, 0x0065, 0x0052, 0x0065, 0x006E, 0x0064, 0x0065, 0x0072, 0x0000,
    ];
    status = PcRegisterSubdevice(device_object, wave_render_name.as_ptr(), render_port);
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: PcRegisterSubdevice WaveRT FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }

    DbgPrint("Leyline: Registering TopoRender Port\n\0".as_ptr());
    let mut render_topo_port: *mut u8 = core::ptr::null_mut();
    status = PcNewPort(&mut render_topo_port, &CLSID_PortTopology);
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: PcNewPort Topology FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }
    let render_topo_miniport_com = MiniportTopologyCom::new(false);
    let render_topo_miniport_ptr = Box::into_raw(render_topo_miniport_com) as *mut u8;
    (*dev_ext).render_topo_miniport = render_topo_miniport_ptr as *mut MiniportTopologyCom;

    let vtable = *(render_topo_port as *const *const *const u8);
    let init_ptr = *vtable.add(3);
    let topo_init_fn: PortInitFn = core::mem::transmute(init_ptr);

    status = topo_init_fn(
        render_topo_port,
        device_object,
        _irp,
        render_topo_miniport_ptr,
        core::ptr::null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: Port Init Topology FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }
    let topo_render_name: [u16; 11] = [
        0x0054, 0x006F, 0x0070, 0x006F, 0x0052, 0x0065, 0x006E, 0x0064, 0x0065, 0x0072, 0x0000,
    ];
    status = PcRegisterSubdevice(device_object, topo_render_name.as_ptr(), render_topo_port);
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: PcRegisterSubdevice Topology FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }

    status = PcRegisterPhysicalConnection(
        device_object,
        render_port,
        KSPIN_WAVE_BRIDGE,
        render_topo_port,
        KSPIN_TOPO_BRIDGE,
    );
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: PcRegisterPhysicalConnection Render FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }

    DbgPrint("Leyline: Registering WaveCapture Port\n\0".as_ptr());
    let mut capture_port: *mut u8 = core::ptr::null_mut();
    status = PcNewPort(&mut capture_port, &CLSID_PortWaveRT);
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: PcNewPort Capture WaveRT FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }
    let capture_miniport_com = MiniportWaveRTCom::new(true, dev_ext);
    let capture_miniport_ptr = Box::into_raw(capture_miniport_com) as *mut u8;
    (*dev_ext).capture_miniport = capture_miniport_ptr as *mut MiniportWaveRTCom;

    let vtable = *(capture_port as *const *const *const u8);
    let init_ptr = *vtable.add(3);
    let capture_init_fn: PortInitFn = core::mem::transmute(init_ptr);

    status = capture_init_fn(
        capture_port,
        device_object,
        _irp,
        capture_miniport_ptr,
        core::ptr::null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: Port Init Capture WaveRT FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }

    let wave_capture_name: [u16; 12] = [
        0x0057, 0x0061, 0x0076, 0x0065, 0x0043, 0x0061, 0x0070, 0x0074, 0x0075, 0x0072, 0x0065,
        0x0000,
    ];
    status = PcRegisterSubdevice(device_object, wave_capture_name.as_ptr(), capture_port);
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: PcRegisterSubdevice Capture WaveRT FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }

    DbgPrint("Leyline: Registering TopoCapture Port\n\0".as_ptr());
    let mut capture_topo_port: *mut u8 = core::ptr::null_mut();
    status = PcNewPort(&mut capture_topo_port, &CLSID_PortTopology);
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: PcNewPort Capture Topology FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }
    let capture_topo_miniport_com = MiniportTopologyCom::new(true);
    let capture_topo_miniport_ptr = Box::into_raw(capture_topo_miniport_com) as *mut u8;
    (*dev_ext).capture_topo_miniport = capture_topo_miniport_ptr as *mut MiniportTopologyCom;

    let vtable = *(capture_topo_port as *const *const *const u8);
    let init_ptr = *vtable.add(3);
    let capture_topo_init_fn: PortInitFn = core::mem::transmute(init_ptr);

    status = capture_topo_init_fn(
        capture_topo_port,
        device_object,
        _irp,
        capture_topo_miniport_ptr,
        core::ptr::null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: Port Init Capture Topology FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }
    let topo_capture_name: [u16; 12] = [
        0x0054, 0x006F, 0x0070, 0x006F, 0x0043, 0x0061, 0x0070, 0x0074, 0x0075, 0x0072, 0x0065,
        0x0000,
    ];
    status = PcRegisterSubdevice(device_object, topo_capture_name.as_ptr(), capture_topo_port);
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: PcRegisterSubdevice Capture Topology FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    }
    status = PcRegisterPhysicalConnection(
        device_object,
        capture_topo_port,
        KSPIN_TOPO_LINEOUT,
        capture_port,
        KSPIN_WAVE_BRIDGE,
    );
    if status != STATUS_SUCCESS {
        DbgPrint(
            "Leyline: PcRegisterPhysicalConnection Capture FAILED (0x%08X)\n\0".as_ptr(),
            status,
        );
        return status;
    } else {
        DbgPrint("Leyline: StartDevice COMPLETED SUCCESSFULLY\n\0".as_ptr());
    }
    status
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
