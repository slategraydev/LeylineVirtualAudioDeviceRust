// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// CAPTURE PATH DESCRIPTORS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use wdk_sys::*;

use super::common::*;
use crate::constants::*;
use crate::stream::{PCCONNECTION, PCFILTER_DESCRIPTOR, PCPIN_DESCRIPTOR};
use crate::audio::KSPIN_DESCRIPTOR;

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_PINS: [PCPIN_DESCRIPTOR; 2] = [
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 4,
        MaxFilterInstanceCount: 4,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null_mut(),
            DataRangesCount: 2,
            DataRanges: WAVE_DATARANGES.as_ptr() as *const *mut crate::stream::KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_SINK as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null_mut(),
            MediumsCount: 0,
            Mediums: core::ptr::null_mut(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut crate::stream::KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
];

#[link_section = ".rdata"]
pub static TOPO_CAPTURE_PINS: [PCPIN_DESCRIPTOR; 2] = [
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null_mut(),
            MediumsCount: 0,
            Mediums: core::ptr::null_mut(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut crate::stream::KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32,
            Category: &KSNODETYPE_MICROPHONE as *const GUID,
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null_mut(),
            MediumsCount: 0,
            Mediums: core::ptr::null_mut(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut crate::stream::KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_NONE as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
];

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromNodePin: KSPIN_WAVE_BRIDGE,
    ToNode: PCFILTER_NODE,
    ToNodePin: KSPIN_WAVE_SINK,
}];

#[link_section = ".rdata"]
pub static TOPO_CAPTURE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromNodePin: 0,
    ToNode: PCFILTER_NODE,
    ToNodePin: 1,
}];

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_CATEGORIES: [GUID; 3] = [
    KSCATEGORY_AUDIO_GUID,
    KSCATEGORY_CAPTURE_GUID,
    KSCATEGORY_REALTIME_GUID,
];

#[link_section = ".rdata"]
pub static TOPO_CAPTURE_CATEGORIES: [GUID; 2] = [KSCATEGORY_AUDIO_GUID, KSCATEGORY_TOPOLOGY_GUID];

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &WAVE_FILTER_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: WAVE_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: WAVE_CAPTURE_CONNECTIONS.as_ptr(),
    CategoryCount: 3,
    Categories: WAVE_CAPTURE_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static TOPO_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &TOPO_FILTER_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: TOPO_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: TOPO_CAPTURE_CONNECTIONS.as_ptr(),
    CategoryCount: 2,
    Categories: TOPO_CAPTURE_CATEGORIES.as_ptr(),
};
