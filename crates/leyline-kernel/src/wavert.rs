// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

use crate::adapter::{DeviceExtension, MiniportWaveRTStreamCom};
use crate::constants::*;
use crate::descriptors::*;
use crate::stream::{MiniportWaveRTStream, TimeSource, KSDATARANGE, PCFILTER_DESCRIPTOR};
use crate::vtables::*;
use alloc::boxed::Box;
use wdk_sys::ntddk::*;
use wdk_sys::*;

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

    pub fn init(&mut self, _ua: PVOID, _rl: PVOID, _p: PVOID) -> NTSTATUS {
        unsafe {
            DbgPrint(
                "Leyline: MiniportWaveRT::Init (capture=%d)\n\0".as_ptr() as *const i8,
                self.is_capture as i32,
            );
        }
        self.is_initialized = true;
        STATUS_SUCCESS
    }

    pub fn new_stream(&mut self, _pin: u32, _cap: bool, _fmt: PVOID) -> *mut MiniportWaveRTStream {
        unsafe {
            DbgPrint("Leyline: MiniportWaveRT::NewStream\n\0".as_ptr() as *const i8);
        }
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

pub unsafe extern "system" fn miniport_query_interface(
    this: *mut u8,
    iid: *const GUID,
    out: *mut *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTCom;
    if iid.is_null() || out.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    if crate::is_equal_guid(iid, &IID_IMiniportWaveRT)
        || crate::is_equal_guid(iid, &IID_IUnknown)
        || crate::is_equal_guid(iid, &IID_IMiniport)
    {
        (*com_obj).ref_count += 1;
        *out = this;
        STATUS_SUCCESS
    } else {
        *out = core::ptr::null_mut();
        STATUS_NOINTERFACE
    }
}

pub unsafe extern "system" fn miniport_add_ref(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportWaveRTCom;
    (*com_obj).ref_count += 1;
    (*com_obj).ref_count
}

pub unsafe extern "system" fn miniport_release(this: *mut u8) -> u32 {
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
    let com_obj = this as *mut MiniportWaveRTCom;
    DbgPrint(
        "Leyline: MiniportWaveRT::GetDescription (capture=%d)\n\0".as_ptr() as *const i8,
        (*com_obj).inner.is_capture as i32,
    );
    let description = out_description as *mut *const PCFILTER_DESCRIPTOR;
    if (*com_obj).inner.is_capture {
        *description = &WAVE_CAPTURE_FILTER_DESCRIPTOR;
    } else {
        *description = &WAVE_RENDER_FILTER_DESCRIPTOR;
    }
    STATUS_SUCCESS
}

pub unsafe extern "system" fn miniport_data_range_intersection(
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
    unsafe {
        if !crate::is_equal_guid(&(*ks_range).MajorFormat, &KSDATAFORMAT_TYPE_AUDIO) {
            return STATUS_NO_MATCH;
        }
        if !crate::is_equal_guid(&(*ks_range).Specifier, &KSDATAFORMAT_SPECIFIER_WAVEFORMATEX) {
            return STATUS_NO_MATCH;
        }
        let is_pcm = crate::is_equal_guid(&(*ks_range).SubFormat, &KSDATAFORMAT_SUBTYPE_PCM);
        let is_float =
            crate::is_equal_guid(&(*ks_range).SubFormat, &KSDATAFORMAT_SUBTYPE_IEEE_FLOAT);
        if !is_pcm && !is_float {
            return STATUS_NO_MATCH;
        }

        let format_size = core::mem::size_of::<crate::stream::KSDATAFORMAT_WAVEFORMATEX>() as u32;
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
            ((*result).WaveFormatEx.nChannels * (*result).WaveFormatEx.wBitsPerSample / 8) as u16;
        (*result).WaveFormatEx.nAvgBytesPerSec =
            (*result).WaveFormatEx.nSamplesPerSec * (*result).WaveFormatEx.nBlockAlign as u32;
        (*result).WaveFormatEx.cbSize = 0;

        if !actual_data_format_cb.is_null() {
            *actual_data_format_cb = format_size;
        }
    }
    STATUS_SUCCESS
}

pub unsafe extern "system" fn miniport_init(
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

pub unsafe extern "system" fn miniport_get_device_description(
    this: *mut u8,
    description: *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportWaveRTCom;
    (*com_obj)
        .inner
        .get_device_description(description as PDEVICE_DESCRIPTION)
}

pub unsafe extern "system" fn miniport_new_stream(
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
