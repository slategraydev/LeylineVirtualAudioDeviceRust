// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

// First std/core/alloc.
use alloc::boxed::Box;
use core::ptr::null_mut;

// Second, external crates.
use wdk_sys::ntddk::*;
use wdk_sys::*;

// Then current crate.
use crate::constants::*;
use crate::descriptors::*;
use crate::stream::PCFILTER_DESCRIPTOR;
use crate::vtables::*;

pub struct MiniportTopology {
    pub is_initialized: u32,
    pub is_capture: u32,
    pub port: PVOID,
}

impl MiniportTopology {
    pub fn new(is_capture: bool) -> Self {
        Self {
            is_initialized: 0,
            is_capture: is_capture as u32,
            port: core::ptr::null_mut(),
        }
    }

    pub fn init(
        &mut self,
        _unknown_adapter: PVOID,
        _resource_list: PVOID,
        port: PVOID,
    ) -> NTSTATUS {
        self.port = port;
        self.is_initialized = 1;
        STATUS_SUCCESS
    }
}

#[repr(C)]
pub struct MiniportTopologyCom {
    pub vtable: *const IMiniportTopologyVTable,
    pub pin_count_vtable: *const IPinCountVTable,
    pub pin_name_vtable: *const IPinNameVTable,
    pub inner: MiniportTopology,
    pub ref_count: u32,
}

#[link_section = ".rdata"]
pub static TOPOLOGY_VTABLE: IMiniportTopologyVTable = IMiniportTopologyVTable {
    base: IUnknownVTable {
        QueryInterface: topology_query_interface,
        AddRef: topology_add_ref,
        Release: topology_release,
    },
    GetDescription: topology_get_description,
    DataRangeIntersection: topology_data_range_intersection,
    Init: topology_init,
};

#[link_section = ".rdata"]
pub static PIN_COUNT_VTABLE: IPinCountVTable = IPinCountVTable {
    base: IUnknownVTable {
        QueryInterface: topology_query_interface,
        AddRef: topology_add_ref,
        Release: topology_release,
    },
    PinCount: topology_pin_count,
};

#[link_section = ".rdata"]
pub static PIN_NAME_VTABLE: IPinNameVTable = IPinNameVTable {
    base: IUnknownVTable {
        QueryInterface: topology_query_interface,
        AddRef: topology_add_ref,
        Release: topology_release,
    },
    GetPinName: topology_get_pin_name,
};

impl MiniportTopologyCom {
    pub fn new(is_capture: bool) -> Box<Self> {
        Box::new(Self {
            vtable: &TOPOLOGY_VTABLE,
            pin_count_vtable: &PIN_COUNT_VTABLE,
            pin_name_vtable: &PIN_NAME_VTABLE,
            inner: MiniportTopology::new(is_capture),
            ref_count: 1,
        })
    }

    /// Recovers the base MiniportTopologyCom pointer from any of its interface pointers.
    ///
    /// # Safety
    /// 'this' must be a valid pointer to one of the VTable fields in MiniportTopologyCom.
    pub unsafe fn from_this(this: *mut u8) -> *mut Self {
        let vtable_ptr = *(this as *mut *const u8);
        if vtable_ptr == &TOPOLOGY_VTABLE as *const _ as *const u8 {
            this as *mut Self
        } else if vtable_ptr == &PIN_COUNT_VTABLE as *const _ as *const u8 {
            (this as usize - 8) as *mut Self
        } else if vtable_ptr == &PIN_NAME_VTABLE as *const _ as *const u8 {
            (this as usize - 16) as *mut Self
        } else {
            // Fallback: assume primary interface if unknown.
            this as *mut Self
        }
    }
}

/// QueryInterface callback for Topology miniport.
///
/// # Safety
/// Standard COM-like QueryInterface. Parameters must be valid pointers.
pub unsafe extern "system" fn topology_query_interface(
    this: *mut u8,
    iid: *const GUID,
    out: *mut *mut u8,
) -> NTSTATUS {
    // Validate parameters
    if this.is_null() || iid.is_null() || out.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let com_obj = MiniportTopologyCom::from_this(this);

    // Log the requested interface GUID (Simple string only)
    DbgPrint(c"LeylineTopo: QueryInterface called\n".as_ptr());

    // Check for known interfaces and log
    if crate::is_equal_guid(iid, &IID_IMiniportTopology)
        || crate::is_equal_guid(iid, &IID_IUnknown)
        || crate::is_equal_guid(iid, &IID_IMiniport)
    {
        DbgPrint(c"LeylineTopo: QueryInterface -> IMiniportTopology (ACCEPTED)\n".as_ptr());
        *out = &((*com_obj).vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IPinCount) {
        DbgPrint(c"LeylineTopo: QueryInterface -> IPinCount (ACCEPTED)\n".as_ptr());
        *out = &((*com_obj).pin_count_vtable) as *const _ as *mut u8;
    } else if crate::is_equal_guid(iid, &IID_IPinName) {
        DbgPrint(c"LeylineTopo: QueryInterface -> IPinName (ACCEPTED)\n".as_ptr());
        *out = &((*com_obj).pin_name_vtable) as *const _ as *mut u8;
    } else {
        DbgPrint(
            c"LeylineTopo: QueryInterface -> Other IID: {%08x-...} (REJECTED)\n".as_ptr(),
            (*iid).Data1,
        );
        *out = null_mut();
        return STATUS_NOINTERFACE;
    }

    (*com_obj).ref_count += 1;
    STATUS_SUCCESS
}

// ... existing code ...

/// PinCount callback for Topology miniport.
pub unsafe extern "system" fn topology_pin_count(
    this: *mut u8,
    pin_id: u32,
    _filter_necessary: *mut u32,
    _filter_current: *mut u32,
    _filter_possible: *mut u32,
    _global_current: *mut u32,
    _global_possible: *mut u32,
) {
    let _com_obj = MiniportTopologyCom::from_this(this);
    DbgPrint(
        c"LeylineTopo: PinCount called for pin %d\n".as_ptr(),
        pin_id,
    );
}

/// GetPinName callback for Topology miniport.
pub unsafe extern "system" fn topology_get_pin_name(
    this: *mut u8,
    irp: *mut u8,
    pin: *mut u8,
    data: *mut u8,
) -> NTSTATUS {
    let com_obj = MiniportTopologyCom::from_this(this);
    DbgPrint(c"LeylineTopo: GetPinName called\n".as_ptr());

    if irp.is_null() || pin.is_null() || data.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    let pin_id = *(pin as *const u32);
    let out_unicode = data as *mut UNICODE_STRING;

    // Use KSNODETYPE to determine name
    let name_prefix = if (*com_obj).inner.is_capture != 0 {
        if pin_id == 0 {
            "Leyline Capture Pin"
        } else {
            "Leyline Capture Bridge"
        }
    } else {
        if pin_id == 1 {
            "Leyline Render Pin"
        } else {
            "Leyline Render Bridge"
        }
    };

    // Allocate buffer for the string (PortCls will free this)
    let buffer_len = (name_prefix.len() * 2) as u16;
    let buffer = ExAllocatePool2(
        POOL_FLAG_PAGED,
        buffer_len as u64,
        u32::from_be_bytes(*b"LLPN"),
    ) as *mut u16;

    if buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    for (i, c) in name_prefix.encode_utf16().enumerate() {
        *buffer.add(i) = c;
    }

    (*out_unicode).Length = buffer_len;
    (*out_unicode).MaximumLength = buffer_len;
    (*out_unicode).Buffer = buffer;

    STATUS_SUCCESS
}

/// AddRef callback for Topology miniport.
///
/// # Safety
/// Standard COM-like AddRef. Parameters must be valid pointers.
pub unsafe extern "system" fn topology_add_ref(this: *mut u8) -> u32 {
    let com_obj = MiniportTopologyCom::from_this(this);
    (*com_obj).ref_count += 1;
    (*com_obj).ref_count
}

/// Release callback for Topology miniport.
///
/// # Safety
/// Standard COM-like Release. Parameters must be valid pointers.
pub unsafe extern "system" fn topology_release(this: *mut u8) -> u32 {
    let com_obj = MiniportTopologyCom::from_this(this);
    (*com_obj).ref_count -= 1;
    let count = (*com_obj).ref_count;
    if count == 0 {
        drop(Box::from_raw(com_obj));
    }
    count
}

/// GetDescription callback for Topology miniport.
///
/// # Safety
/// Standard PortCls callback. Parameters must be valid pointers.
pub unsafe extern "system" fn topology_get_description(
    this: *mut u8,
    out_description: *mut u8,
) -> NTSTATUS {
    DbgPrint(c"LeylineTopo: GetDescription called\n".as_ptr());

    if this.is_null() || out_description.is_null() {
        DbgPrint(c"LeylineTopo: GetDescription - NULL parameter\n".as_ptr());
        return STATUS_INVALID_PARAMETER;
    }

    let com_obj = MiniportTopologyCom::from_this(this);
    let description = out_description as *mut *const PCFILTER_DESCRIPTOR;

    // Validate the descriptor before returning
    let descriptor_ptr = if (*com_obj).inner.is_capture != 0 {
        &TOPO_CAPTURE_FILTER_DESCRIPTOR
    } else {
        &TOPO_RENDER_FILTER_DESCRIPTOR
    };

    // Log descriptor info
    DbgPrint(c"LeylineTopo: Returning descriptor\n".as_ptr());

    *description = descriptor_ptr;
    DbgPrint(c"LeylineTopo: GetDescription SUCCESS\n".as_ptr());
    STATUS_SUCCESS
}

/// DataRangeIntersection callback for Topology miniport.
///
/// # Safety
/// Standard PortCls callback. Parameters must be valid pointers.
pub unsafe extern "system" fn topology_data_range_intersection(
    _this: *mut u8,
    pin_id: u32,
    data_range: *mut u8,
    matching_data_range: *mut u8,
    data_format_cb: u32,
    data_format: *mut u8,
    actual_data_format_cb: *mut u32,
) -> NTSTATUS {
    // Topology filters have 2 pins: bridge (0) and lineout/mic (1)
    if pin_id > 1 {
        return STATUS_INVALID_PARAMETER;
    }

    // Check if data_range is valid
    if data_range.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    // For topology analog bridge pins, we accept any analog data range
    // Use the bridge data range from descriptors
    let bridge_range = &crate::descriptors::BRIDGE_DATARANGE as *const crate::stream::KSDATARANGE;

    // If matching_data_range is requested, copy the bridge data range
    if !matching_data_range.is_null() {
        let src = bridge_range as *const u8;
        let dst = matching_data_range;
        let size = core::mem::size_of::<crate::stream::KSDATARANGE>();
        if size <= core::mem::size_of_val(&(*data_range)) {
            core::ptr::copy_nonoverlapping(src, dst, size);
        }
    }

    // If data format is requested, create a minimal data format
    if !data_format.is_null()
        && data_format_cb >= core::mem::size_of::<crate::stream::KSDATAFORMAT>() as u32
    {
        let format = data_format as *mut crate::stream::KSDATAFORMAT;
        *format = crate::stream::KSDATAFORMAT {
            FormatSize: core::mem::size_of::<crate::stream::KSDATAFORMAT>() as u32,
            Flags: 0,
            SampleSize: 0,
            Reserved: 0,
            MajorFormat: crate::constants::KSDATAFORMAT_TYPE_AUDIO,
            SubFormat: crate::constants::KSDATAFORMAT_SUBTYPE_ANALOG,
            Specifier: crate::constants::KSDATAFORMAT_SPECIFIER_NONE_GUID,
        };

        if !actual_data_format_cb.is_null() {
            *actual_data_format_cb = core::mem::size_of::<crate::stream::KSDATAFORMAT>() as u32;
        }

        return STATUS_SUCCESS;
    } else if !data_format.is_null() {
        // Buffer too small
        if !actual_data_format_cb.is_null() {
            *actual_data_format_cb = core::mem::size_of::<crate::stream::KSDATAFORMAT>() as u32;
        }
        return STATUS_BUFFER_TOO_SMALL;
    }

    STATUS_SUCCESS
}

/// Init callback for Topology miniport.
///
/// # Safety
/// Standard PortCls callback. Parameters must be valid pointers.
pub unsafe extern "system" fn topology_init(
    this: *mut u8,
    unknown_adapter: *mut u8,
    resource_list: *mut u8,
    port: *mut u8,
) -> NTSTATUS {
    DbgPrint(c"LeylineTopo: Init called\n".as_ptr());

    if this.is_null() {
        DbgPrint(c"LeylineTopo: Init - this is NULL\n".as_ptr());
        return STATUS_INVALID_PARAMETER;
    }

    DbgPrint(c"LeylineTopo: Init parameters validated\n".as_ptr());

    let com_obj = MiniportTopologyCom::from_this(this);
    let status = (*com_obj).inner.init(
        unknown_adapter as PVOID,
        resource_list as PVOID,
        port as PVOID,
    );

    if status == STATUS_SUCCESS {
        DbgPrint(c"LeylineTopo: Init SUCCESS\n".as_ptr());
    } else {
        DbgPrint(c"LeylineTopo: Init FAILED\n".as_ptr());
    }

    status
}
