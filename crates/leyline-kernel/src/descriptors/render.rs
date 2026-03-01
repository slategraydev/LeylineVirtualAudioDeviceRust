// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// RENDER PATH DESCRIPTORS
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use wdk_sys::*;

use crate::descriptors::common::*;
use crate::stream::{PCCONNECTION, PCFILTER_DESCRIPTOR, PCPIN_DESCRIPTOR};

#[link_section = ".rdata"]
pub static WAVE_RENDER_PINS: [PCPIN_DESCRIPTOR; 2] = [
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 4,
        MaxFilterInstanceCount: 4,
        MinFilterInstanceCount: 1,
        AutomationTable: &PIN_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 2,
            DataRanges: WAVE_DATARANGES.as_ptr() as *const *mut crate::stream::KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
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
pub static TOPO_RENDER_PINS: [PCPIN_DESCRIPTOR; 2] = [
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
            Category: &KSNODETYPE_SPEAKER as *const GUID,
            Name: core::ptr::null_mut(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
];

#[link_section = ".rdata"]
pub static WAVE_RENDER_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromNodePin: crate::constants::KSPIN_WAVE_SINK,
    ToNode: PCFILTER_NODE,
    ToNodePin: crate::constants::KSPIN_WAVE_BRIDGE,
}];

#[link_section = ".rdata"]
pub static TOPO_RENDER_CONNECTIONS: [PCCONNECTION; 3] = [
    PCCONNECTION {
        FromNode: PCFILTER_NODE,
        FromNodePin: crate::constants::KSPIN_TOPO_BRIDGE,
        ToNode: 0,
        ToNodePin: 1,
    },
    PCCONNECTION {
        FromNode: 0,
        FromNodePin: 0,
        ToNode: 1,
        ToNodePin: 1,
    },
    PCCONNECTION {
        FromNode: 1,
        FromNodePin: 0,
        ToNode: PCFILTER_NODE,
        ToNodePin: crate::constants::KSPIN_TOPO_LINEOUT,
    },
];

#[link_section = ".rdata"]
pub static TOPO_RENDER_NODES: [crate::stream::PCNODE_DESCRIPTOR; 2] = [
    crate::stream::PCNODE_DESCRIPTOR {
        Flags: 0,
        AutomationTable: &VOLUME_AUTOMATION_TABLE,
        Type: &KSNODETYPE_VOLUME,
        Name: &KSAUDFNAME_MASTER_VOLUME,
    },
    crate::stream::PCNODE_DESCRIPTOR {
        Flags: 0,
        AutomationTable: &MUTE_AUTOMATION_TABLE,
        Type: &KSNODETYPE_MUTE,
        Name: &KSAUDFNAME_MASTER_MUTE,
    },
];

#[link_section = ".rdata"]
pub static WAVE_RENDER_CATEGORIES: [GUID; 3] = [
    crate::constants::KSCATEGORY_AUDIO_GUID,
    crate::constants::KSCATEGORY_RENDER_GUID,
    crate::constants::KSCATEGORY_REALTIME_GUID,
];

#[link_section = ".rdata"]
pub static TOPO_RENDER_CATEGORIES: [GUID; 2] = [
    crate::constants::KSCATEGORY_AUDIO_GUID,
    crate::constants::KSCATEGORY_TOPOLOGY_GUID,
];

#[link_section = ".rdata"]
pub static WAVE_RENDER_FILTER_PROPERTIES: [crate::stream::PCPROPERTY_ITEM; 9] = [
    crate::stream::PCPROPERTY_ITEM {
        Set: &KSPROPSETID_GENERAL as *const GUID,
        Id: KSPROPERTY_GENERAL_COMPONENTID,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(component_id_handler),
    },
    crate::stream::PCPROPERTY_ITEM {
        Set: &KSPROPSETID_PIN as *const GUID,
        Id: crate::constants::KSPROPERTY_PIN_PROPOSEDATAFORMAT,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(proposed_format_handler),
    },
    crate::stream::PCPROPERTY_ITEM {
        Set: &KSPROPSETID_PIN as *const GUID,
        Id: crate::constants::KSPROPERTY_PIN_PROPOSEDATAFORMAT2,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(proposed_format_handler),
    },
    crate::stream::PCPROPERTY_ITEM {
        Set: &KSPROPSETID_JACK as *const GUID,
        Id: KSPROPERTY_JACK_DESCRIPTION,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(jack_description_handler),
    },
    crate::stream::PCPROPERTY_ITEM {
        Set: &KSPROPSETID_JACK as *const GUID,
        Id: KSPROPERTY_JACK_DESCRIPTION2,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(jack_description_handler),
    },
    crate::stream::PCPROPERTY_ITEM {
        Set: &KSPROPSETID_AUDIOEFFECTSDISCOVERY as *const GUID,
        Id: KSPROPERTY_AUDIOEFFECTSDISCOVERY_EFFECTSLIST,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(audio_effects_discovery_handler),
    },
    crate::stream::PCPROPERTY_ITEM {
        Set: &KSPROPSETID_AUDIOMODULE as *const GUID,
        Id: KSPROPERTY_AUDIOMODULE_DESCRIPTORS,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(audio_module_handler),
    },
    crate::stream::PCPROPERTY_ITEM {
        Set: &KSPROPSETID_AUDIOMODULE as *const GUID,
        Id: KSPROPERTY_AUDIOMODULE_COMMAND,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_SET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(audio_module_handler),
    },
    crate::stream::PCPROPERTY_ITEM {
        Set: &KSPROPSETID_AUDIOMODULE as *const GUID,
        Id: KSPROPERTY_AUDIOMODULE_NOTIFICATION_DEVICE_ID,
        Flags: KSPROPERTY_TYPE_GET | KSPROPERTY_TYPE_BASICSUPPORT,
        Handler: Some(audio_module_handler),
    },
];

#[link_section = ".rdata"]
pub static WAVE_RENDER_FILTER_AUTOMATION_TABLE: crate::stream::PCAUTOMATION_TABLE =
    crate::stream::PCAUTOMATION_TABLE {
        PropertyItemSize: core::mem::size_of::<crate::stream::PCPROPERTY_ITEM>() as u32,
        PropertyCount: 9,
        Properties: WAVE_RENDER_FILTER_PROPERTIES.as_ptr(),
        MethodItemSize: 0,
        MethodCount: 0,
        Methods: core::ptr::null(),
        EventItemSize: 0,
        EventCount: 0,
        Events: core::ptr::null(),
        Reserved: 0,
    };

#[link_section = ".rdata"]
pub static WAVE_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &WAVE_RENDER_FILTER_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: WAVE_RENDER_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: WAVE_RENDER_CONNECTIONS.as_ptr(),
    CategoryCount: 3,
    Categories: WAVE_RENDER_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static TOPO_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &TOPO_FILTER_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: TOPO_RENDER_PINS.as_ptr(),
    NodeSize: core::mem::size_of::<crate::stream::PCNODE_DESCRIPTOR>() as u32,
    NodeCount: 2,
    Nodes: TOPO_RENDER_NODES.as_ptr(),
    ConnectionCount: 3,
    Connections: TOPO_RENDER_CONNECTIONS.as_ptr(),
    CategoryCount: 2,
    Categories: TOPO_RENDER_CATEGORIES.as_ptr(),
};
