// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

// First std/core/alloc.
use alloc::boxed::Box;
use core::mem::size_of;
use core::ptr::null_mut;

// Second, external crates.
use wdk_sys::ntddk::*;
use wdk_sys::*;

// Then current crate.
use crate::adapter::{DeviceExtension, MiniportWaveRTStreamCom};
use crate::constants::*;
use crate::descriptors::*;
use crate::stream::{MiniportWaveRTStream, TimeSource, KSDATARANGE, PCFILTER_DESCRIPTOR};
use crate::vtables::*;

pub struct MiniportWaveRT {
    pub max_pci_bar: u32,
    pub is_initialized: bool,
    pub is_capture: bool,
    pub streams: [Option<Box<MiniportWaveRTStream>>; 4],
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

    pub fn init(
        &mut self,
        _unknown_adapter: PVOID,
        _resource_list: PVOID,
        _port: PVOID,
    ) -> NTSTATUS {
        unsafe {
            DbgPrint(
                c"LeylineWaveRT: Init (capture=%d)\n".as_ptr(),
                self.is_capture as i32,
            );
        }
        self.is_initialized = true;
        STATUS_SUCCESS
    }

    /// Creates a new WaveRT stream.
    pub unsafe fn new_stream(
        &mut self,
        _pin_id: u32,
        is_capture: bool,
        format: PVOID,
    ) -> *mut MiniportWaveRTStream {
        DbgPrint(c"LeylineWaveRT: NewStream\n".as_ptr());
        if !self.is_initialized {
            return null_mut();
        }

        for slot in self.streams.iter_mut() {
            if slot.is_none() {
                *slot = Some(Box::new(MiniportWaveRTStream::new(
                    format,
                    is_capture,
                    Box::new(crate::stream::KernelTimeSource) as Box<dyn TimeSource>,
                    self.device_extension as *mut u8,
                )));
                return slot.as_mut().unwrap().as_mut() as *mut MiniportWaveRTStream;
            }
        }

        null_mut()
    }

    pub fn get_device_description(&self, _device_description: PDEVICE_DESCRIPTION) -> NTSTATUS {
        STATUS_SUCCESS
    }
}

#[repr(C)]
pub struct MiniportWaveRTCom {
    pub vtable: *const IMiniportWaveRTVTable,
    pub output_stream_vtable: *const IMiniportWaveRTOutputStreamVTable,
    pub input_stream_vtable: *const IMiniportWaveRTInputStreamVTable,
    pub pin_count_vtable: *const IPinCountVTable,
    pub inner: MiniportWaveRT,
    pub ref_count: u32,
}

#[link_section = ".rdata"]
pub static MINIPORT_VTABLE: IMiniportWaveRTVTable = IMiniportWaveRTVTable {
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

#[link_section = ".rdata"]
pub static OUTPUT_STREAM_VTABLE: IMiniportWaveRTOutputStreamVTable =
    IMiniportWaveRTOutputStreamVTable {
        base: IUnknownVTable {
            QueryInterface: miniport_query_interface,
            AddRef: miniport_add_ref,
            Release: miniport_release,
        },
        SetWritePacket: wavert_set_write_packet,
        GetOutputStreamPresentationPosition: wavert_get_output_stream_presentation_position,
        GetPacketCount: wavert_get_packet_count,
    };

#[link_section = ".rdata"]
pub static INPUT_STREAM_VTABLE: IMiniportWaveRTInputStreamVTable =
    IMiniportWaveRTInputStreamVTable {
        base: IUnknownVTable {
            QueryInterface: miniport_query_interface,
            AddRef: miniport_add_ref,
            Release: miniport_release,
        },
        GetReadPacket: wavert_get_read_packet,
    };

#[link_section = ".rdata"]
pub static WAVERT_PIN_COUNT_VTABLE: IPinCountVTable = IPinCountVTable {
    base: IUnknownVTable {
        QueryInterface: miniport_query_interface,
        AddRef: miniport_add_ref,
        Release: miniport_release,
    },
    PinCount: wavert_pin_count,
};

impl MiniportWaveRTCom {
    pub fn new(is_capture: bool, device_extension: *mut DeviceExtension) -> Box<Self> {
        Box::new(Self {
            vtable: &MINIPORT_VTABLE,
            output_stream_vtable: &OUTPUT_STREAM_VTABLE,
            input_stream_vtable: &INPUT_STREAM_VTABLE,
            pin_count_vtable: &WAVERT_PIN_COUNT_VTABLE,
            inner: MiniportWaveRT::new(is_capture, device_extension),
            ref_count: 1,
        })
    }
}

// ============================================================================
// Miniport VTable Callbacks
// ============================================================================

pub unsafe extern "system" fn miniport_query_interface(
    this: *mut u8,
    iid: *const GUID,
    out: *mut *mut u8,
) -> NTSTATUS {
    if this.is_null() || iid.is_null() || out.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let com_obj = this as *mut MiniportWaveRTCom;

    if crate::is_equal_guid(iid, &IID_IMiniportWaveRT)
        || crate::is_equal_guid(iid, &IID_IUnknown)
        || crate::is_equal_guid(iid, &IID_IMiniport)
    {
        DbgPrint(c"LeylineWaveRT: QueryInterface -> IMiniportWaveRT\n".as_ptr());
        *out = &((*com_obj).vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IMiniportWaveRTOutputStream) {
        DbgPrint(c"LeylineWaveRT: QueryInterface -> IMiniportWaveRTOutputStream\n".as_ptr());
        *out = &((*com_obj).output_stream_vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IMiniportWaveRTInputStream) {
        DbgPrint(c"LeylineWaveRT: QueryInterface -> IMiniportWaveRTInputStream\n".as_ptr());
        *out = &((*com_obj).input_stream_vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IPinCount) {
        DbgPrint(c"LeylineWaveRT: QueryInterface -> IPinCount\n".as_ptr());
        *out = &((*com_obj).pin_count_vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IPortClsStreamResourceManager)
        || crate::is_equal_guid(iid, &IID_IPortClsStreamResourceManager2)
        || crate::is_equal_guid(iid, &IID_IAdapterPnpManagement)
        || crate::is_equal_guid(iid, &IID_IMiniportPnpNotify)
        || crate::is_equal_guid(iid, &IID_IMiniportAudioSignalProcessing)
    {
        DbgPrint(c"LeylineWaveRT: QueryInterface -> base interface\n".as_ptr());
        *out = &((*com_obj).vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IMiniportAudioEngineNode) {
        DbgPrint(
            c"LeylineWaveRT: QueryInterface -> IMiniportAudioEngineNode (REJECTED)\n".as_ptr(),
        );
        *out = null_mut();
        return STATUS_NOINTERFACE;
    } else {
        DbgPrint(
            c"LeylineWaveRT: QueryInterface -> Unknown IID: {%08x-...}\n".as_ptr(),
            (*iid).Data1,
        );
        *out = null_mut();
        return STATUS_NOINTERFACE;
    }

    (*com_obj).ref_count += 1;
    STATUS_SUCCESS
}

pub unsafe extern "system" fn miniport_add_ref(this: *mut u8) -> u32 {
    if this.is_null() {
        return 0;
    }
    let com_obj = this as *mut MiniportWaveRTCom;
    (*com_obj).ref_count += 1;
    (*com_obj).ref_count
}

pub unsafe extern "system" fn miniport_release(this: *mut u8) -> u32 {
    if this.is_null() {
        return 0;
    }
    let com_obj = this as *mut MiniportWaveRTCom;
    (*com_obj).ref_count -= 1;
    let count = (*com_obj).ref_count;
    if count == 0 {
        drop(Box::from_raw(com_obj));
    }
    count
}

pub unsafe extern "system" fn miniport_get_description(
    this: *mut u8,
    out_description: *mut u8,
) -> NTSTATUS {
    if this.is_null() || out_description.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let com_obj = this as *mut MiniportWaveRTCom;
    DbgPrint(c"LeylineWaveRT: GetDescription called\n".as_ptr());
    let description = out_description as *mut *const PCFILTER_DESCRIPTOR;
    if (*com_obj).inner.is_capture {
        *description = &WAVE_CAPTURE_FILTER_DESCRIPTOR;
    } else {
        *description = &WAVE_RENDER_FILTER_DESCRIPTOR;
    }
    STATUS_SUCCESS
}

pub unsafe extern "system" fn miniport_data_range_intersection(
    this: *mut u8,
    _pin_id: u32,
    data_range: *mut u8,
    _matching_range: *mut u8,
    data_format_cb: u32,
    data_format: *mut u8,
    actual_data_format_cb: *mut u32,
) -> NTSTATUS {
    if this.is_null() || data_range.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    DbgPrint(c"LeylineWaveRT: DataRangeIntersection called\n".as_ptr());

    let ks_range = data_range as *const KSDATARANGE;
    if !crate::is_equal_guid(&(*ks_range).MajorFormat, &KSDATAFORMAT_TYPE_AUDIO) {
        return STATUS_NO_MATCH;
    }
    if !crate::is_equal_guid(&(*ks_range).Specifier, &KSDATAFORMAT_SPECIFIER_WAVEFORMATEX) {
        return STATUS_NO_MATCH;
    }

    let is_pcm = crate::is_equal_guid(&(*ks_range).SubFormat, &KSDATAFORMAT_SUBTYPE_PCM);
    let is_float = crate::is_equal_guid(&(*ks_range).SubFormat, &KSDATAFORMAT_SUBTYPE_IEEE_FLOAT);
    if !is_pcm && !is_float {
        return STATUS_NO_MATCH;
    }

    let format_size = size_of::<crate::stream::KSDATAFORMAT_WAVEFORMATEX>() as u32;
    if data_format_cb == 0 {
        if !actual_data_format_cb.is_null() {
            *actual_data_format_cb = format_size;
        }
        return STATUS_BUFFER_OVERFLOW;
    }
    if data_format_cb < format_size {
        return STATUS_BUFFER_TOO_SMALL;
    }

    let result = data_format as *mut crate::stream::KSDATAFORMAT_WAVEFORMATEX;
    (*result).DataFormat.FormatSize = format_size;
    (*result).DataFormat.MajorFormat = KSDATAFORMAT_TYPE_AUDIO;
    (*result).DataFormat.SubFormat = (*ks_range).SubFormat;
    (*result).DataFormat.Specifier = KSDATAFORMAT_SPECIFIER_WAVEFORMATEX;

    (*result).WaveFormatEx.wFormatTag = if is_pcm { 1 } else { 3 };
    (*result).WaveFormatEx.nChannels = 2;
    (*result).WaveFormatEx.nSamplesPerSec = 48000;
    (*result).WaveFormatEx.wBitsPerSample = if is_pcm { 16 } else { 32 };
    (*result).WaveFormatEx.nBlockAlign =
        (*result).WaveFormatEx.nChannels * (*result).WaveFormatEx.wBitsPerSample / 8;
    (*result).WaveFormatEx.nAvgBytesPerSec =
        (*result).WaveFormatEx.nSamplesPerSec * (*result).WaveFormatEx.nBlockAlign as u32;
    (*result).WaveFormatEx.cbSize = 0;

    if !actual_data_format_cb.is_null() {
        *actual_data_format_cb = format_size;
    }
    STATUS_SUCCESS
}

pub unsafe extern "system" fn miniport_init(
    this: *mut u8,
    unknown_adapter: *mut u8,
    resource_list: *mut u8,
    port: *mut u8,
) -> NTSTATUS {
    if this.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let com_obj = this as *mut MiniportWaveRTCom;
    (*com_obj).inner.init(
        unknown_adapter as PVOID,
        resource_list as PVOID,
        port as PVOID,
    )
}

pub unsafe extern "system" fn miniport_get_device_description(
    this: *mut u8,
    description: *mut u8,
) -> NTSTATUS {
    if this.is_null() || description.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let com_obj = this as *mut MiniportWaveRTCom;
    (*com_obj)
        .inner
        .get_device_description(description as PDEVICE_DESCRIPTION)
}

pub unsafe extern "system" fn miniport_new_stream(
    this: *mut u8,
    stream: *mut *mut u8,
    _port: *mut u8,
    pin_id: u32,
    capture: bool,
    format: *mut u8,
) -> NTSTATUS {
    if this.is_null() || stream.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    let com_obj = this as *mut MiniportWaveRTCom;
    DbgPrint(c"LeylineWaveRT: NewStream called\n".as_ptr());
    let stream_ptr = (*com_obj)
        .inner
        .new_stream(pin_id, capture, format as PVOID);
    if stream_ptr.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let stream_com = MiniportWaveRTStreamCom::new(stream_ptr);
    *stream = Box::into_raw(stream_com) as *mut u8;
    STATUS_SUCCESS
}

pub unsafe extern "system" fn wavert_pin_count(
    _this: *mut u8,
    pin_id: u32,
    _filter_necessary: *mut u32,
    _filter_current: *mut u32,
    _filter_possible: *mut u32,
    _global_current: *mut u32,
    _global_possible: *mut u32,
) {
    DbgPrint(
        c"LeylineWaveRT: PinCount called for pin %d\n".as_ptr(),
        pin_id,
    );
}

pub unsafe extern "system" fn wavert_set_write_packet(
    _this: *mut u8,
    _p: u32,
    _f: u32,
    _l: u32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
pub unsafe extern "system" fn wavert_get_output_stream_presentation_position(
    _this: *mut u8,
    _p: *mut u64,
    _c: *mut u64,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
pub unsafe extern "system" fn wavert_get_packet_count(_this: *mut u8, _c: *mut u32) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
pub unsafe extern "system" fn wavert_get_read_packet(
    _this: *mut u8,
    _p: *mut u32,
    _f: *mut u32,
    _c: *mut u64,
    _m: *mut i32,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
