// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

// Second, external crates.
use wdk_sys::GUID;

// Then current crate.
use crate::constants::*;
use crate::stream::{
    KSDATAFORMAT, KSDATARANGE, KSDATARANGE_AUDIO, KSPIN_DESCRIPTOR, PCAUTOMATION_TABLE,
    PCCONNECTION, PCFILTER_DESCRIPTOR, PCPIN_DESCRIPTOR,
};

#[repr(C)]
#[derive(Copy, Clone)]
#[allow(non_snake_case)]
pub struct KSINTERFACE_STANDARD {
    pub InterfaceId: GUID,
    pub Reserved: u32,
    pub Version: u32,
}

// Simplified: Just use the GUID for now and a raw pointer if needed, or implement the struct.
// KSPIN_INTERFACE is usually just a KSIDENTIFIER.
#[repr(C)]
#[allow(non_snake_case)]
pub struct KSIDENTIFIER {
    pub Set: GUID,
    pub Id: u32,
    pub Flags: u32,
}

#[link_section = ".rdata"]
pub static KSINTERFACES: [KSIDENTIFIER; 1] = [KSIDENTIFIER {
    Set: KSINTERFACESETID_STANDARD,
    Id: 0, // KSINTERFACE_STANDARD_STREAMING
    Flags: 0,
}];

#[repr(transparent)]
pub struct SyncPtr<T>(pub *const T);
unsafe impl<T> Sync for SyncPtr<T> {}

// ============================================================================
// Data Ranges
// ============================================================================

#[link_section = ".rdata"]
pub static PCM_DATARANGE: KSDATARANGE_AUDIO = KSDATARANGE_AUDIO {
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

#[link_section = ".rdata"]
pub static FLOAT_DATARANGE: KSDATARANGE_AUDIO = KSDATARANGE_AUDIO {
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

#[link_section = ".rdata"]
pub static BRIDGE_DATARANGE: KSDATARANGE = KSDATARANGE {
    FormatSize: core::mem::size_of::<KSDATARANGE>() as u32,
    Flags: 0,
    SampleSize: 0,
    Reserved: 0,
    MajorFormat: KSDATAFORMAT_TYPE_AUDIO,
    SubFormat: KSDATAFORMAT_SUBTYPE_PCM, // ✅ Changed from ANALOG
    Specifier: KSDATAFORMAT_SPECIFIER_NONE_GUID,
};

#[link_section = ".rdata"]
pub static WAVE_DATARANGES: [SyncPtr<KSDATARANGE>; 2] = [
    SyncPtr(&PCM_DATARANGE.DataRange as *const KSDATARANGE),
    SyncPtr(&FLOAT_DATARANGE.DataRange as *const KSDATARANGE),
];

#[link_section = ".rdata"]
pub static BRIDGE_DATARANGES: [SyncPtr<KSDATARANGE>; 1] =
    [SyncPtr(&BRIDGE_DATARANGE as *const KSDATARANGE)];

#[link_section = ".rdata"]
pub static MINIMAL_AUTOMATION_TABLE: PCAUTOMATION_TABLE = PCAUTOMATION_TABLE {
    PropertyItemSize: 0,
    PropertyCount: 0,
    Properties: core::ptr::null(),
    MethodItemSize: 0,
    MethodCount: 0,
    Methods: core::ptr::null(),
    EventItemSize: 0,
    EventCount: 0,
    Events: core::ptr::null(),
    Reserved: 0,
};

// ============================================================================
// Pin Descriptors
// ============================================================================

#[link_section = ".rdata"]
pub static WAVE_RENDER_PINS: [PCPIN_DESCRIPTOR; 2] = [
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 4,
        MaxFilterInstanceCount: 4,
        MinFilterInstanceCount: 1,
        AutomationTable: &MINIMAL_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 2,
            DataRanges: WAVE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_SINK as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &MINIMAL_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_BRIDGE as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID, // ✅ Keep standard for bridge
            Name: core::ptr::null(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
];

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_PINS: [PCPIN_DESCRIPTOR; 2] = [
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 4,
        MaxFilterInstanceCount: 4,
        MinFilterInstanceCount: 1,
        AutomationTable: &MINIMAL_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 2,
            DataRanges: WAVE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_SINK as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &MINIMAL_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_BRIDGE as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID, // ✅ Keep standard for bridge
            Name: core::ptr::null(),
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
        AutomationTable: &MINIMAL_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_BRIDGE as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &MINIMAL_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_BRIDGE as i32,
            Category: &KSNODETYPE_SPEAKER as *const GUID, // ✅ Edge pin
            Name: core::ptr::null(),
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
        AutomationTable: &MINIMAL_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_IN as i32,
            Communication: KSPIN_COMMUNICATION_BRIDGE as i32,
            Category: &KSNODETYPE_MICROPHONE as *const GUID, // ✅ Edge pin
            Name: core::ptr::null(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 1,
        MaxFilterInstanceCount: 1,
        MinFilterInstanceCount: 1,
        AutomationTable: &MINIMAL_AUTOMATION_TABLE,
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 1,
            Interfaces: KSINTERFACES.as_ptr() as *const core::ffi::c_void,
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 1,
            DataRanges: BRIDGE_DATARANGES.as_ptr() as *const *mut KSDATAFORMAT,
            DataFlow: KSPIN_DATAFLOW_OUT as i32,
            Communication: KSPIN_COMMUNICATION_BRIDGE as i32,
            Category: &KSCATEGORY_AUDIO_GUID as *const GUID,
            Name: core::ptr::null(),
            Reserved: 0,
            Reserved2: 0,
        },
    },
];

// ============================================================================
// Connections & Categories
// ============================================================================

#[link_section = ".rdata"]
pub static WAVE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromNodePin: KSPIN_WAVE_SINK,
    ToNode: PCFILTER_NODE,
    ToNodePin: KSPIN_WAVE_BRIDGE,
}];

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromNodePin: KSPIN_WAVE_BRIDGE,
    ToNode: PCFILTER_NODE,
    ToNodePin: KSPIN_WAVE_SINK,
}];

#[link_section = ".rdata"]
pub static TOPO_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromNodePin: KSPIN_TOPO_BRIDGE,
    ToNode: PCFILTER_NODE,
    ToNodePin: KSPIN_TOPO_LINEOUT,
}];

#[link_section = ".rdata"]
pub static TOPO_CAPTURE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromNodePin: 0, // MICROPHONE pin (index 0, KSNODETYPE_MICROPHONE)
    ToNode: PCFILTER_NODE,
    ToNodePin: 1, // BRIDGE pin (index 1)
}];

#[link_section = ".rdata"]
pub static TOPO_FILTER_CATEGORIES: [GUID; 2] = [KSCATEGORY_AUDIO_GUID, KSCATEGORY_TOPOLOGY_GUID];
#[link_section = ".rdata"]
pub static WAVE_RENDER_CATEGORIES: [GUID; 3] = [
    KSCATEGORY_AUDIO_GUID,
    KSCATEGORY_RENDER_GUID,
    KSCATEGORY_REALTIME_GUID,
];
#[link_section = ".rdata"]
pub static WAVE_CAPTURE_CATEGORIES: [GUID; 3] = [
    KSCATEGORY_AUDIO_GUID,
    KSCATEGORY_CAPTURE_GUID,
    KSCATEGORY_REALTIME_GUID,
];

// ============================================================================
// Filter Descriptors
// ============================================================================

#[link_section = ".rdata"]
pub static WAVE_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &MINIMAL_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: WAVE_RENDER_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: WAVE_CONNECTIONS.as_ptr(),
    CategoryCount: 3,
    Categories: WAVE_RENDER_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &MINIMAL_AUTOMATION_TABLE,
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
pub static TOPO_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &MINIMAL_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: TOPO_RENDER_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: TOPO_CONNECTIONS.as_ptr(),
    CategoryCount: 2,
    Categories: TOPO_FILTER_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static TOPO_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: &MINIMAL_AUTOMATION_TABLE,
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: TOPO_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: TOPO_CAPTURE_CONNECTIONS.as_ptr(),
    CategoryCount: 2,
    Categories: TOPO_FILTER_CATEGORIES.as_ptr(),
};
