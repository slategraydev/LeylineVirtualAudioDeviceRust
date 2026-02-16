// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

use crate::constants::*;
use crate::stream::{
    KSDATARANGE, KSDATARANGE_AUDIO, KSPIN_DESCRIPTOR, PCCONNECTION, PCFILTER_DESCRIPTOR,
    PCPIN_DESCRIPTOR,
};
use wdk_sys::GUID;

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
    SubFormat: KSDATAFORMAT_SUBTYPE_ANALOG,
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

// ============================================================================
// Pin Descriptors
// ============================================================================

#[link_section = ".rdata"]
pub static WAVE_RENDER_PINS: [PCPIN_DESCRIPTOR; 2] = [
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 4, // MAX_STREAMS
        MaxFilterInstanceCount: 4,
        MinFilterInstanceCount: 1,
        AutomationTable: core::ptr::null(),
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null(),
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 2,
            DataRanges: WAVE_DATARANGES.as_ptr() as *const *const KSDATARANGE,
            DataFlow: KSPIN_DATAFLOW_IN,
            Communication: KSPIN_COMMUNICATION_SINK,
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
            DataFlow: KSPIN_DATAFLOW_OUT,
            Communication: KSPIN_COMMUNICATION_BRIDGE,
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
        },
    },
];

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_PINS: [PCPIN_DESCRIPTOR; 2] = [
    PCPIN_DESCRIPTOR {
        MaxGlobalInstanceCount: 4, // MAX_STREAMS
        MaxFilterInstanceCount: 4,
        MinFilterInstanceCount: 1,
        AutomationTable: core::ptr::null(),
        KsPinDescriptor: KSPIN_DESCRIPTOR {
            InterfacesCount: 0,
            Interfaces: core::ptr::null(),
            MediumsCount: 0,
            Mediums: core::ptr::null(),
            DataRangesCount: 2,
            DataRanges: WAVE_DATARANGES.as_ptr() as *const *const KSDATARANGE,
            DataFlow: KSPIN_DATAFLOW_OUT,
            Communication: KSPIN_COMMUNICATION_SOURCE,
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
            DataFlow: KSPIN_DATAFLOW_IN,
            Communication: KSPIN_COMMUNICATION_BRIDGE,
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
        },
    },
];

#[link_section = ".rdata"]
pub static TOPO_RENDER_PINS: [PCPIN_DESCRIPTOR; 2] = [
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
            DataFlow: KSPIN_DATAFLOW_IN,
            Communication: KSPIN_COMMUNICATION_BRIDGE,
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
            DataFlow: KSPIN_DATAFLOW_OUT,
            Communication: KSPIN_COMMUNICATION_NONE,
            Category: &KSNODETYPE_SPEAKER,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
        },
    },
];

#[link_section = ".rdata"]
pub static TOPO_CAPTURE_PINS: [PCPIN_DESCRIPTOR; 2] = [
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
            DataFlow: KSPIN_DATAFLOW_IN,
            Communication: KSPIN_COMMUNICATION_NONE,
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
            DataFlow: KSPIN_DATAFLOW_OUT,
            Communication: KSPIN_COMMUNICATION_BRIDGE,
            Category: &KSCATEGORY_AUDIO_GUID,
            Name: core::ptr::null(),
            Reserved: core::ptr::null_mut(),
        },
    },
];

// ============================================================================
// Connections & Categories
// ============================================================================

#[link_section = ".rdata"]
pub static WAVE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromPin: KSPIN_WAVE_SINK,
    ToNode: PCFILTER_NODE,
    ToPin: KSPIN_WAVE_BRIDGE,
}];

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromPin: KSPIN_WAVE_BRIDGE,
    ToNode: PCFILTER_NODE,
    ToPin: KSPIN_WAVE_SINK,
}];

#[link_section = ".rdata"]
pub static TOPO_CONNECTIONS: [PCCONNECTION; 1] = [PCCONNECTION {
    FromNode: PCFILTER_NODE,
    FromPin: KSPIN_TOPO_BRIDGE,
    ToNode: PCFILTER_NODE,
    ToPin: KSPIN_TOPO_LINEOUT,
}];

#[link_section = ".rdata"]
pub static TOPO_FILTER_CATEGORIES: [GUID; 2] = [KSCATEGORY_AUDIO_GUID, KSCATEGORY_TOPOLOGY_GUID];
#[link_section = ".rdata"]
pub static WAVE_RENDER_CATEGORIES: [GUID; 2] = [KSCATEGORY_AUDIO_GUID, KSCATEGORY_RENDER_GUID];
#[link_section = ".rdata"]
pub static WAVE_CAPTURE_CATEGORIES: [GUID; 2] = [KSCATEGORY_AUDIO_GUID, KSCATEGORY_CAPTURE_GUID];

// ============================================================================
// Filter Descriptors
// ============================================================================

#[link_section = ".rdata"]
pub static WAVE_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: core::ptr::null(),
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: WAVE_RENDER_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: WAVE_CONNECTIONS.as_ptr(),
    CategoryCount: 2,
    Categories: WAVE_RENDER_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static WAVE_CAPTURE_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: core::ptr::null(),
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: WAVE_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: WAVE_CAPTURE_CONNECTIONS.as_ptr(),
    CategoryCount: 2,
    Categories: WAVE_CAPTURE_CATEGORIES.as_ptr(),
};

#[link_section = ".rdata"]
pub static TOPO_RENDER_FILTER_DESCRIPTOR: PCFILTER_DESCRIPTOR = PCFILTER_DESCRIPTOR {
    Version: 0,
    AutomationTable: core::ptr::null(),
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
    AutomationTable: core::ptr::null(),
    PinSize: core::mem::size_of::<PCPIN_DESCRIPTOR>() as u32,
    PinCount: 2,
    Pins: TOPO_CAPTURE_PINS.as_ptr(),
    NodeSize: 0,
    NodeCount: 0,
    Nodes: core::ptr::null(),
    ConnectionCount: 1,
    Connections: TOPO_CONNECTIONS.as_ptr(),
    CategoryCount: 2,
    Categories: TOPO_FILTER_CATEGORIES.as_ptr(),
};
