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
use wdk_sys::*;

// Then current crate.
use crate::constants::*;
use crate::descriptors::*;
use crate::stream::PCFILTER_DESCRIPTOR;
use crate::vtables::*;

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

#[repr(C)]
pub struct MiniportTopologyCom {
    pub vtable: *const IMiniportTopologyVTable,
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

impl MiniportTopologyCom {
    pub fn new(is_capture: bool) -> Box<Self> {
        Box::new(Self {
            vtable: &TOPOLOGY_VTABLE,
            inner: MiniportTopology::new(is_capture),
            ref_count: 1,
        })
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
    let com_obj = this as *mut MiniportTopologyCom;
    if crate::is_equal_guid(iid, &IID_IMiniportTopology)
        || crate::is_equal_guid(iid, &IID_IUnknown)
        || crate::is_equal_guid(iid, &IID_IMiniport)
    {
        (*com_obj).ref_count += 1;
        *out = this;
        return STATUS_SUCCESS;
    }

    *out = null_mut();
    STATUS_NOINTERFACE
}

/// AddRef callback for Topology miniport.
///
/// # Safety
/// Standard COM-like AddRef. Parameters must be valid pointers.
pub unsafe extern "system" fn topology_add_ref(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportTopologyCom;
    (*com_obj).ref_count += 1;
    (*com_obj).ref_count
}

/// Release callback for Topology miniport.
///
/// # Safety
/// Standard COM-like Release. Parameters must be valid pointers.
pub unsafe extern "system" fn topology_release(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportTopologyCom;
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
    let com_obj = this as *mut MiniportTopologyCom;
    let description = out_description as *mut *const PCFILTER_DESCRIPTOR;
    if (*com_obj).inner.is_capture {
        *description = &TOPO_CAPTURE_FILTER_DESCRIPTOR;
    } else {
        *description = &TOPO_RENDER_FILTER_DESCRIPTOR;
    }
    STATUS_SUCCESS
}

/// DataRangeIntersection callback for Topology miniport.
///
/// # Safety
/// Standard PortCls callback. Parameters must be valid pointers.
pub unsafe extern "system" fn topology_data_range_intersection(
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
    let com_obj = this as *mut MiniportTopologyCom;
    (*com_obj).inner.init(
        unknown_adapter as PVOID,
        resource_list as PVOID,
        port as PVOID,
    )
}
