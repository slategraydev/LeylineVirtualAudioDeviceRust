// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

use crate::constants::*;
use crate::descriptors::*;
use crate::stream::PCFILTER_DESCRIPTOR;
use crate::vtables::*;
use alloc::boxed::Box;
use wdk_sys::ntddk::*;
use wdk_sys::*;

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

pub unsafe extern "system" fn topology_query_interface(
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

pub unsafe extern "system" fn topology_add_ref(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportTopologyCom;
    (*com_obj).ref_count += 1;
    (*com_obj).ref_count
}

pub unsafe extern "system" fn topology_release(this: *mut u8) -> u32 {
    let com_obj = this as *mut MiniportTopologyCom;
    (*com_obj).ref_count -= 1;
    let count = (*com_obj).ref_count;
    if count == 0 {
        drop(Box::from_raw(com_obj));
    }
    count
}

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

pub unsafe extern "system" fn topology_data_range_intersection(
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

pub unsafe extern "system" fn topology_init(
    this: *mut u8,
    ua: *mut u8,
    rl: *mut u8,
    p: *mut u8,
) -> NTSTATUS {
    let com_obj = this as *mut MiniportTopologyCom;
    (*com_obj).inner.init(ua as PVOID, rl as PVOID, p as PVOID)
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

fn is_equal_guid(a: *const GUID, b: &GUID) -> bool {
    unsafe {
        (*a).Data1 == b.Data1
            && (*a).Data2 == b.Data2
            && (*a).Data3 == b.Data3
            && (*a).Data4 == b.Data4
    }
}
