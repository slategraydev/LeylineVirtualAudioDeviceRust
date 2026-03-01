#![allow(clippy::missing_safety_doc)]

// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// WAVERT MINIPORT & COM INTERFACES
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use alloc::boxed::Box;
use core::mem::size_of;
use core::ptr::null_mut;

use wdk_sys::ntddk::*;
use wdk_sys::*;

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

    /// Create a new WaveRT stream instance.
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
    pub pin_name_vtable: *const IPinNameVTable,
    pub resource_manager_vtable: *const IPortClsStreamResourceManager2VTable,
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

#[link_section = ".rdata"]
pub static WAVERT_PIN_NAME_VTABLE: IPinNameVTable = IPinNameVTable {
    base: IUnknownVTable {
        QueryInterface: miniport_query_interface,
        AddRef: miniport_add_ref,
        Release: miniport_release,
    },
    GetPinName: wavert_get_pin_name,
};

#[link_section = ".rdata"]
pub static RESOURCE_MANAGER_VTABLE: IPortClsStreamResourceManager2VTable =
    IPortClsStreamResourceManager2VTable {
        base: IUnknownVTable {
            QueryInterface: miniport_query_interface,
            AddRef: miniport_add_ref,
            Release: miniport_release,
        },
        AddResource: wavert_add_resource,
        RemoveResource: wavert_remove_resource,
    };

impl MiniportWaveRTCom {
    pub fn new(is_capture: bool, device_extension: *mut DeviceExtension) -> Box<Self> {
        Box::new(Self {
            vtable: &MINIPORT_VTABLE,
            output_stream_vtable: &OUTPUT_STREAM_VTABLE,
            input_stream_vtable: &INPUT_STREAM_VTABLE,
            pin_count_vtable: &WAVERT_PIN_COUNT_VTABLE,
            pin_name_vtable: &WAVERT_PIN_NAME_VTABLE,
            resource_manager_vtable: &RESOURCE_MANAGER_VTABLE,
            inner: MiniportWaveRT::new(is_capture, device_extension),
            ref_count: 1,
        })
    }

    /// Recover the base MiniportWaveRTCom pointer from any of its interface pointers.
    ///
    /// # Safety
    /// 'this' must be a valid pointer to one of the VTable fields in MiniportWaveRTCom.
    pub unsafe fn from_this(this: *mut u8) -> *mut Self {
        let vtable_ptr = *(this as *mut *const u8);
        if vtable_ptr == &MINIPORT_VTABLE as *const _ as *const u8 {
            this as *mut Self
        } else if vtable_ptr == &OUTPUT_STREAM_VTABLE as *const _ as *const u8 {
            (this as usize - 8) as *mut Self
        } else if vtable_ptr == &INPUT_STREAM_VTABLE as *const _ as *const u8 {
            (this as usize - 16) as *mut Self
        } else if vtable_ptr == &WAVERT_PIN_COUNT_VTABLE as *const _ as *const u8 {
            (this as usize - 24) as *mut Self
        } else if vtable_ptr == &WAVERT_PIN_NAME_VTABLE as *const _ as *const u8 {
            (this as usize - 32) as *mut Self
        } else if vtable_ptr == &RESOURCE_MANAGER_VTABLE as *const _ as *const u8 {
            (this as usize - 40) as *mut Self
        } else {
            // Fallback: assume primary interface if unknown.
            this as *mut Self
        }
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Miniport VTable Callbacks
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

// SAFETY: Standard COM QueryInterface.
// 'this' is a pointer to the interface (which is a pointer to the vtable).
// 'out' is a pointer to a void* where we must return the new interface pointer.
pub unsafe extern "system" fn miniport_query_interface(
    this: *mut u8,
    iid: *const GUID,
    out: *mut *mut u8,
) -> NTSTATUS {
    if this.is_null() || iid.is_null() || out.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let com_obj = MiniportWaveRTCom::from_this(this);

    if crate::is_equal_guid(iid, &IID_IMiniportWaveRT)
        || crate::is_equal_guid(iid, &IID_IUnknown)
        || crate::is_equal_guid(iid, &IID_IMiniport)
    {
        DbgPrint(c"LeylineWaveRT: QueryInterface -> IMiniportWaveRT (ACCEPTED)\n".as_ptr());
        *out = &((*com_obj).vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IMiniportWaveRTOutputStream) {
        DbgPrint(
            c"LeylineWaveRT: QueryInterface -> IMiniportWaveRTOutputStream (ACCEPTED)\n".as_ptr(),
        );
        *out = &((*com_obj).output_stream_vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IMiniportWaveRTInputStream) {
        DbgPrint(
            c"LeylineWaveRT: QueryInterface -> IMiniportWaveRTInputStream (ACCEPTED)\n".as_ptr(),
        );
        *out = &((*com_obj).input_stream_vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IPinCount) {
        DbgPrint(c"LeylineWaveRT: QueryInterface -> IPinCount (ACCEPTED)\n".as_ptr());
        *out = &((*com_obj).pin_count_vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IPinName) {
        DbgPrint(c"LeylineWaveRT: QueryInterface -> IPinName (ACCEPTED)\n".as_ptr());
        *out = &((*com_obj).pin_name_vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IPortClsStreamResourceManager)
        || crate::is_equal_guid(iid, &IID_IPortClsStreamResourceManager2)
    {
        DbgPrint(
            c"LeylineWaveRT: QueryInterface -> IPortClsStreamResourceManager2 (ACCEPTED)\n"
                .as_ptr(),
        );
        *out = &((*com_obj).resource_manager_vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IPowerNotify) {
        DbgPrint(
            c"LeylineWaveRT: QueryInterface -> IPowerNotify (REJECTED - NOT IMPLEMENTED)\n"
                .as_ptr(),
        );
        return STATUS_NOINTERFACE;
    } else if crate::is_equal_guid(iid, &IID_IAdapterPowerManagement)
        || crate::is_equal_guid(iid, &IID_IAdapterPowerManagement2)
        || crate::is_equal_guid(iid, &IID_IAdapterPowerManagement3)
    {
        DbgPrint(c"LeylineWaveRT: QueryInterface -> IAdapterPowerManagementX (REJECTED - NOT IMPLEMENTED)\n".as_ptr());
        return STATUS_NOINTERFACE;
    } else if crate::is_equal_guid(iid, &IID_IMiniportAudioSignalProcessing) {
        DbgPrint(c"LeylineWaveRT: QueryInterface -> IMiniportAudioSignalProcessing (REJECTED - NOT IMPLEMENTED)\n".as_ptr());
        return STATUS_NOINTERFACE;
    } else if crate::is_equal_guid(iid, &IID_IMiniportAudioEngineNode) {
        DbgPrint(c"LeylineWaveRT: QueryInterface -> IMiniportAudioEngineNode (REJECTED - NOT IMPLEMENTED)\n".as_ptr());
        return STATUS_NOINTERFACE;
    } else {
        DbgPrint(
            c"LeylineWaveRT: QueryInterface -> UNKNOWN (%08x-%04x-%04x-%02x%02x-%02x%02x%02x%02x%02x%02x)\n"
                .as_ptr(),
            (*iid).Data1,
            (*iid).Data2 as u32,
            (*iid).Data3 as u32,
            (*iid).Data4[0] as u32,
            (*iid).Data4[1] as u32,
            (*iid).Data4[2] as u32,
            (*iid).Data4[3] as u32,
            (*iid).Data4[4] as u32,
            (*iid).Data4[5] as u32,
            (*iid).Data4[6] as u32,
            (*iid).Data4[7] as u32,
        );
        *out = null_mut();
        return STATUS_NOINTERFACE;
    }

    (*com_obj).ref_count += 1;
    STATUS_SUCCESS
}

pub unsafe extern "system" fn wavert_add_resource(
    _this: *mut u8,
    _resource_description: *mut u8,
    _resource_handle: *mut *mut u8,
) -> NTSTATUS {
    DbgPrint(c"LeylineWaveRT: AddResource (STUB)\n".as_ptr());
    STATUS_SUCCESS
}

pub unsafe extern "system" fn wavert_remove_resource(
    _this: *mut u8,
    _resource_handle: *mut u8,
) -> NTSTATUS {
    DbgPrint(c"LeylineWaveRT: RemoveResource (STUB)\n".as_ptr());
    STATUS_SUCCESS
}

pub unsafe extern "system" fn miniport_add_ref(this: *mut u8) -> u32 {
    if this.is_null() {
        return 0;
    }
    let com_obj = MiniportWaveRTCom::from_this(this);
    (*com_obj).ref_count += 1;
    (*com_obj).ref_count
}

pub unsafe extern "system" fn miniport_release(this: *mut u8) -> u32 {
    if this.is_null() {
        return 0;
    }
    let com_obj = MiniportWaveRTCom::from_this(this);
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
    let com_obj = MiniportWaveRTCom::from_this(this);
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
    pin_id: u32,
    data_range: *mut u8,
    _matching_range: *mut u8,
    data_format_cb: u32,
    data_format: *mut u8,
    actual_data_format_cb: *mut u32,
) -> NTSTATUS {
    if this.is_null() || data_range.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let ks_range = data_range as *const KSDATARANGE;
    DbgPrint(
        c"LeylineWaveRT: DataRangeIntersection Pin %d, SubFormat: {%08x-...}\n".as_ptr(),
        pin_id,
        (*ks_range).SubFormat.Data1,
    );

    let _com_obj = MiniportWaveRTCom::from_this(this);
    if !crate::is_equal_guid(&(*ks_range).MajorFormat, &KSDATAFORMAT_TYPE_AUDIO) {
        DbgPrint(c"LeylineWaveRT: -> FAILED: Not Audio\n".as_ptr());
        return STATUS_NO_MATCH;
    }
    if !crate::is_equal_guid(&(*ks_range).Specifier, &KSDATAFORMAT_SPECIFIER_WAVEFORMATEX) {
        DbgPrint(c"LeylineWaveRT: -> FAILED: Not WaveFormatEx Specifier\n".as_ptr());
        return STATUS_NO_MATCH;
    }

    let is_pcm = crate::is_equal_guid(&(*ks_range).SubFormat, &KSDATAFORMAT_SUBTYPE_PCM);
    let is_float = crate::is_equal_guid(&(*ks_range).SubFormat, &KSDATAFORMAT_SUBTYPE_IEEE_FLOAT);
    if !is_pcm && !is_float {
        DbgPrint(c"LeylineWaveRT: -> FAILED: Not PCM or Float\n".as_ptr());
        return STATUS_NO_MATCH;
    }

    let format_size = size_of::<crate::stream::KSDATAFORMAT_WAVEFORMATEX>() as u32;
    if data_format_cb == 0 {
        if !actual_data_format_cb.is_null() {
            *actual_data_format_cb = format_size;
        }
        DbgPrint(c"LeylineWaveRT: -> Buffer size query (40 bytes)\n".as_ptr());
        return STATUS_BUFFER_TOO_SMALL; // PortCls and AEB expect TOO_SMALL for size queries
    }
    if data_format_cb < format_size {
        DbgPrint(c"LeylineWaveRT: -> FAILED: Buffer too small\n".as_ptr());
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
    // CRITICAL: SampleSize must match nBlockAlign per SYSVAD reference.
    // A zero SampleSize can cause AEB to reject the negotiated format.
    (*result).DataFormat.SampleSize = (*result).WaveFormatEx.nBlockAlign as u32;

    if !actual_data_format_cb.is_null() {
        *actual_data_format_cb = format_size;
    }

    DbgPrint(c"LeylineWaveRT: -> SUCCESS: Negotiated 48kHz Stereo\n".as_ptr());
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
    let com_obj = MiniportWaveRTCom::from_this(this);
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
    let com_obj = MiniportWaveRTCom::from_this(this);
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
    let com_obj = MiniportWaveRTCom::from_this(this);
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
    this: *mut u8,
    pin_id: u32,
    filter_necessary: *mut u32,
    filter_current: *mut u32,
    filter_possible: *mut u32,
    global_current: *mut u32,
    global_possible: *mut u32,
) {
    let _com_obj = MiniportWaveRTCom::from_this(this);
    DbgPrint(
        c"LeylineWaveRT: PinCount called for pin %d\n".as_ptr(),
        pin_id,
    );

    if !filter_necessary.is_null() {
        *filter_necessary = 0;
    }
    if !filter_current.is_null() {
        *filter_current = 0;
    }
    if !filter_possible.is_null() {
        // WaveRT pins support multiple instances (usually 4)
        *filter_possible = 4;
    }
    if !global_current.is_null() {
        *global_current = 0;
    }
    if !global_possible.is_null() {
        *global_possible = 4;
    }
}

// Local KSP_PIN definition for safe access
#[repr(C)]
#[allow(non_snake_case)]
struct KSPROPERTY_LOCAL {
    pub Set: GUID,
    pub Id: u32,
    pub Flags: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct KSP_PIN_LOCAL {
    pub Property: KSPROPERTY_LOCAL,
    pub PinId: u32,
    pub Reserved: u32,
}

pub unsafe extern "system" fn wavert_get_pin_name(
    this: *mut u8,
    _irp: *mut u8,
    pin: *mut u8,
    data: *mut u8,
) -> NTSTATUS {
    if pin.is_null() || data.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let com_obj = MiniportWaveRTCom::from_this(this);
    let ksp_pin = pin as *const KSP_PIN_LOCAL;
    let pin_id = (*ksp_pin).PinId;

    DbgPrint(
        c"LeylineWaveRT: GetPinName CALLED for Pin %d (Capture=%d)\n".as_ptr(),
        pin_id,
        (*com_obj).inner.is_capture as i32,
    );

    let out_unicode = data as *mut UNICODE_STRING;

    let name_prefix = if (*com_obj).inner.is_capture {
        if pin_id == 0 {
            "Leyline Capture Pin"
        } else {
            "Leyline Capture Bridge"
        }
    } else if pin_id == 0 {
        "Leyline Render Pin"
    } else {
        "Leyline Render Bridge"
    };

    let chars: alloc::vec::Vec<u16> = name_prefix
        .encode_utf16()
        .chain(core::iter::once(0))
        .collect();
    let buffer_len = (chars.len() * 2) as u16;

    let buffer = ExAllocatePool2(
        POOL_FLAG_PAGED,
        buffer_len as u64,
        u32::from_be_bytes(*b"LLWP"),
    ) as *mut u16;

    if buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(chars.as_ptr(), buffer, chars.len());

    (*out_unicode).Length = (name_prefix.len() * 2) as u16;
    (*out_unicode).MaximumLength = buffer_len;
    (*out_unicode).Buffer = buffer;

    DbgPrint(c"LeylineWaveRT: GetPinName SUCCESS: %ls\n".as_ptr(), buffer);
    STATUS_SUCCESS
}

pub unsafe extern "system" fn wavert_set_write_packet(
    this: *mut u8,
    packet_number: u32,
    _flags: u32,
    _eos_packet_length: u32,
) -> NTSTATUS {
    let _com_obj = MiniportWaveRTCom::from_this(this);
    DbgPrint(
        c"LeylineWaveRT: SetWritePacket %d\n".as_ptr(),
        packet_number,
    );
    STATUS_NOT_IMPLEMENTED
}

pub unsafe extern "system" fn wavert_get_output_stream_presentation_position(
    this: *mut u8,
    _presentation_position: *mut u64,
    _performance_counter: *mut u64,
) -> NTSTATUS {
    let _com_obj = MiniportWaveRTCom::from_this(this);
    DbgPrint(c"LeylineWaveRT: GetPresentationPosition\n".as_ptr());
    STATUS_NOT_IMPLEMENTED
}

pub unsafe extern "system" fn wavert_get_packet_count(
    this: *mut u8,
    _packet_count: *mut u32,
) -> NTSTATUS {
    let _com_obj = MiniportWaveRTCom::from_this(this);
    DbgPrint(c"LeylineWaveRT: GetPacketCount\n".as_ptr());
    STATUS_NOT_IMPLEMENTED
}

pub unsafe extern "system" fn wavert_get_read_packet(
    this: *mut u8,
    _packet_number: *mut u32,
    _flags: *mut u32,
    _performance_counter: *mut u64,
    _more_data: *mut i32,
) -> NTSTATUS {
    let _com_obj = MiniportWaveRTCom::from_this(this);
    DbgPrint(c"LeylineWaveRT: GetReadPacket\n".as_ptr());
    STATUS_NOT_IMPLEMENTED
}

