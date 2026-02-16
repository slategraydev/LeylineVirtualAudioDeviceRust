// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

use wdk_sys::GUID;

// ============================================================================
// PortCls Class & Interface IDs
// ============================================================================

#[allow(non_upper_case_globals)]
pub const CLSID_PortWaveRT: GUID = GUID {
    Data1: 0xCC9BE57A,
    Data2: 0xEB9E,
    Data3: 0x42B4,
    Data4: [0x94, 0xFC, 0x0C, 0xAD, 0x3D, 0xBC, 0xE7, 0xFA],
};

#[allow(non_upper_case_globals)]
pub const IID_IPortWaveRT: GUID = GUID {
    Data1: 0x339FF909,
    Data2: 0x68A9,
    Data3: 0x4310,
    Data4: [0xB0, 0x9B, 0x27, 0x4E, 0x96, 0xEE, 0x4C, 0xBD],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportWaveRT: GUID = GUID {
    Data1: 0x0F9FC4D6,
    Data2: 0x6061,
    Data3: 0x4F3C,
    Data4: [0xB1, 0xFC, 0x07, 0x5E, 0x35, 0xF7, 0x96, 0x0A],
};

#[allow(non_upper_case_globals)]
pub const IID_IPortWaveRTStream: GUID = GUID {
    Data1: 0x1809CE5A,
    Data2: 0x64BC,
    Data3: 0x4E62,
    Data4: [0xBD, 0x7D, 0x95, 0xBC, 0xE4, 0x3D, 0xE3, 0x93],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportWaveRTStream: GUID = GUID {
    Data1: 0x000AC9AB,
    Data2: 0xFAAB,
    Data3: 0x4F3D,
    Data4: [0x94, 0x55, 0x6F, 0xF8, 0x30, 0x6A, 0x74, 0xA0],
};

#[allow(non_upper_case_globals)]
pub const CLSID_PortTopology: GUID = GUID {
    Data1: 0xB4C90A32,
    Data2: 0x5791,
    Data3: 0x11D0,
    Data4: [0x86, 0xF9, 0x00, 0xA0, 0xC9, 0x11, 0xB5, 0x44],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportTopology: GUID = GUID {
    Data1: 0xB4C90A31,
    Data2: 0x5791,
    Data3: 0x11D0,
    Data4: [0x86, 0xF9, 0x00, 0xA0, 0xC9, 0x11, 0xB5, 0x44],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniport: GUID = GUID {
    Data1: 0xB4C90A24,
    Data2: 0x5791,
    Data3: 0x11D0,
    Data4: [0x86, 0xF9, 0x00, 0xA0, 0xC9, 0x11, 0xB5, 0x44],
};

#[allow(non_upper_case_globals)]
pub const IID_IUnknown: GUID = GUID {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

// ============================================================================
// KS Formats & Specifiers
// ============================================================================

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_TYPE_AUDIO: GUID = GUID {
    Data1: 0x73647561,
    Data2: 0x0000,
    Data3: 0x0010,
    Data4: [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
};

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_SUBTYPE_PCM: GUID = GUID {
    Data1: 0x00000001,
    Data2: 0x0000,
    Data3: 0x0010,
    Data4: [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
};

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_SUBTYPE_IEEE_FLOAT: GUID = GUID {
    Data1: 0x00000003,
    Data2: 0x0000,
    Data3: 0x0010,
    Data4: [0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71],
};

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_SUBTYPE_ANALOG: GUID = GUID {
    Data1: 0x6DBA3190,
    Data2: 0x67BD,
    Data3: 0x11CF,
    Data4: [0xA0, 0xF7, 0x00, 0x20, 0xAF, 0xD1, 0x56, 0xE4],
};

#[allow(non_upper_case_globals)]
pub const KSDATAFORMAT_SPECIFIER_WAVEFORMATEX: GUID = GUID {
    Data1: 0x05589F81,
    Data2: 0xC356,
    Data3: 0x11CE,
    Data4: [0xBF, 0x01, 0x00, 0xAA, 0x00, 0x55, 0x59, 0x5A],
};

pub const KSDATAFORMAT_SPECIFIER_NONE_GUID: GUID = GUID {
    Data1: 0x0F6417D6,
    Data2: 0xC318,
    Data3: 0x11D0,
    Data4: [0xA4, 0x3F, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

// ============================================================================
// Filter & Pin Categories
// ============================================================================

#[allow(non_upper_case_globals)]
pub const KSCATEGORY_AUDIO_GUID: GUID = GUID {
    Data1: 0x6994AD04,
    Data2: 0x93EF,
    Data3: 0x11D0,
    Data4: [0xA3, 0xCC, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub const KSCATEGORY_RENDER_GUID: GUID = GUID {
    Data1: 0x65E8773E,
    Data2: 0x8F56,
    Data3: 0x11D0,
    Data4: [0xA3, 0xB9, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub const KSCATEGORY_CAPTURE_GUID: GUID = GUID {
    Data1: 0x65E8773D,
    Data2: 0x8F56,
    Data3: 0x11D0,
    Data4: [0xA3, 0xB9, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub const KSCATEGORY_TOPOLOGY_GUID: GUID = GUID {
    Data1: 0xDDA54A40,
    Data2: 0x1E4C,
    Data3: 0x11D1,
    Data4: [0xA0, 0x50, 0x40, 0x57, 0x05, 0xC1, 0x00, 0x00],
};

#[allow(non_upper_case_globals)]
pub const KSNODETYPE_SPEAKER: GUID = GUID {
    Data1: 0xDFF21CE1,
    Data2: 0xF70F,
    Data3: 0x11D0,
    Data4: [0xB9, 0x17, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub const KSNODETYPE_MICROPHONE: GUID = GUID {
    Data1: 0xDFF21BE1,
    Data2: 0xF70F,
    Data3: 0x11D0,
    Data4: [0xB9, 0x17, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

// ============================================================================
// Pin Indices & IDs
// ============================================================================

pub const PCFILTER_NODE: u32 = !0u32;

// KSPIN_DATAFLOW
pub const KSPIN_DATAFLOW_IN: u32 = 1;
pub const KSPIN_DATAFLOW_OUT: u32 = 2;

// KSPIN_COMMUNICATION
pub const KSPIN_COMMUNICATION_NONE: u32 = 0;
pub const KSPIN_COMMUNICATION_SINK: u32 = 1;
pub const KSPIN_COMMUNICATION_SOURCE: u32 = 2;
pub const KSPIN_COMMUNICATION_BOTH: u32 = 3;
pub const KSPIN_COMMUNICATION_BRIDGE: u32 = 4;

pub const KSPIN_WAVE_SINK: u32 = 0;
pub const KSPIN_WAVE_BRIDGE: u32 = 1;
pub const KSPIN_TOPO_BRIDGE: u32 = 0;
pub const KSPIN_TOPO_LINEOUT: u32 = 1;
