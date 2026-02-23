// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

// Second, external crates.
use wdk_sys::*;

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
pub const IID_IMiniportWaveRTOutputStream: GUID = GUID {
    Data1: 0x831FC7BC,
    Data2: 0x6347,
    Data3: 0x44BC,
    Data4: [0xB4, 0x7B, 0xC0, 0xC6, 0x57, 0xB5, 0xBF, 0x73],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportWaveRTInputStream: GUID = GUID {
    Data1: 0xCD8E756A,
    Data2: 0x5FC7,
    Data3: 0x4624,
    Data4: [0x98, 0x4B, 0x2A, 0xF0, 0x29, 0x25, 0xB9, 0x1F],
};

#[allow(non_upper_case_globals)]
pub const IID_IPortClsStreamResourceManager: GUID = GUID {
    Data1: 0xE1CD9915,
    Data2: 0xCAB1,
    Data3: 0x4103,
    Data4: [0xBB, 0x2F, 0x7D, 0xC0, 0x9C, 0x9B, 0xE9, 0x42],
};

#[allow(non_upper_case_globals)]
pub const IID_IPortClsStreamResourceManager2: GUID = GUID {
    Data1: 0x0D500BAE,
    Data2: 0xD565,
    Data3: 0x469D,
    Data4: [0xA0, 0xE2, 0xF2, 0x83, 0x76, 0x0D, 0x71, 0x48],
};

#[allow(non_upper_case_globals)]
pub const IID_IAdapterPnpManagement: GUID = GUID {
    Data1: 0x706F2368,
    Data2: 0x4086,
    Data3: 0x47F5,
    Data4: [0xB9, 0x13, 0x57, 0xB7, 0x6E, 0xED, 0x1A, 0x32],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportPnpNotify: GUID = GUID {
    Data1: 0x6B735365,
    Data2: 0x9487,
    Data3: 0x464C,
    Data4: [0x93, 0xE3, 0xFA, 0x2C, 0x63, 0x91, 0xD5, 0xA4],
};

#[allow(non_upper_case_globals)]
pub const IID_IPinCount: GUID = GUID {
    Data1: 0x5DADB7DC,
    Data2: 0xA2CB,
    Data3: 0x4540,
    Data4: [0xA4, 0xA8, 0x42, 0x5E, 0xE4, 0xAE, 0x90, 0x51],
};

#[allow(non_upper_case_globals)]
pub const IID_IPinName: GUID = GUID {
    Data1: 0x29CC9AB1,
    Data2: 0xE89D,
    Data3: 0x413C,
    Data4: [0xB6, 0xB2, 0xF6, 0xD5, 0x00, 0x05, 0xD0, 0x63],
};

#[allow(non_upper_case_globals)]
pub const IID_IPowerNotify: GUID = GUID {
    Data1: 0x3DD648B8,
    Data2: 0x969F,
    Data3: 0x11D1,
    Data4: [0x95, 0xA9, 0x00, 0xC0, 0x4F, 0xB9, 0x25, 0xD3],
};

#[allow(non_upper_case_globals)]
pub const IID_IAdapterPowerManagement: GUID = GUID {
    Data1: 0x793417D0,
    Data2: 0x35FE,
    Data3: 0x11D1,
    Data4: [0xAD, 0x08, 0x00, 0xA0, 0xC9, 0x0A, 0xB1, 0xB0],
};

#[allow(non_upper_case_globals)]
pub const IID_IAdapterPowerManagement2: GUID = GUID {
    Data1: 0xE0F92E5D,
    Data2: 0x67F5,
    Data3: 0x48EE,
    Data4: [0xB5, 0x7A, 0x7D, 0x1E, 0x90, 0xC5, 0xF4, 0x3D],
};

#[allow(non_upper_case_globals)]
pub const IID_IAdapterPowerManagement3: GUID = GUID {
    Data1: 0xA8C7303E,
    Data2: 0xF80C,
    Data3: 0x4BC9,
    Data4: [0xB2, 0xE3, 0xFB, 0x2D, 0x08, 0xBE, 0x92, 0x0F],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportAudioEngineNode: GUID = GUID {
    Data1: 0x2EBF536C,
    Data2: 0xEF57,
    Data3: 0x4C64,
    Data4: [0xBE, 0xDC, 0x25, 0xC1, 0xA6, 0xD6, 0x68, 0xE6],
};

#[allow(non_upper_case_globals)]
pub const IID_IMiniportAudioSignalProcessing: GUID = GUID {
    Data1: 0xB532678C,
    Data2: 0xBE50,
    Data3: 0x472D,
    Data4: [0x99, 0x73, 0x8A, 0x6F, 0x16, 0x59, 0x49, 0x89],
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
pub const IID_IPortTopology: GUID = GUID {
    Data1: 0xB4C90A30,
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
pub const IID_IPort: GUID = GUID {
    Data1: 0xB4C90A25,
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

#[allow(non_upper_case_globals)]
pub const GUID_NULL: GUID = GUID {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
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
pub static KSCATEGORY_AUDIO_GUID: GUID = GUID {
    Data1: 0x6994AD04,
    Data2: 0x93EF,
    Data3: 0x11D0,
    Data4: [0xA3, 0xCC, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub static KSCATEGORY_RENDER_GUID: GUID = GUID {
    Data1: 0x65E8773E,
    Data2: 0x8F56,
    Data3: 0x11D0,
    Data4: [0xA3, 0xB9, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub static KSCATEGORY_CAPTURE_GUID: GUID = GUID {
    Data1: 0x65E8773D,
    Data2: 0x8F56,
    Data3: 0x11D0,
    Data4: [0xA3, 0xB9, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub static KSCATEGORY_TOPOLOGY_GUID: GUID = GUID {
    Data1: 0xDDA54A40,
    Data2: 0x1E4C,
    Data3: 0x11D1,
    Data4: [0xA0, 0x50, 0x40, 0x57, 0x05, 0xC1, 0x00, 0x00],
};

#[allow(non_upper_case_globals)]
pub static KSCATEGORY_REALTIME_GUID: GUID = GUID {
    Data1: 0xEB115FFC,
    Data2: 0x10C8,
    Data3: 0x4964,
    Data4: [0x83, 0x1D, 0x6D, 0xCB, 0x02, 0xE6, 0xF2, 0x3F],
};

// ============================================================================
// KS Interface & Property Sets
// ============================================================================

#[allow(non_upper_case_globals)]
pub const KSINTERFACESETID_STANDARD: GUID = GUID {
    Data1: 0x1A8766A0,
    Data2: 0x62CE,
    Data3: 0x11CF,
    Data4: [0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00],
};

#[allow(non_upper_case_globals)]
pub static KSPROPSETID_PIN: GUID = GUID {
    Data1: 0x8C134960,
    Data2: 0x51AD,
    Data3: 0x11CF,
    Data4: [0x87, 0x8A, 0x94, 0xF8, 0x01, 0xC1, 0x00, 0x00],
};

#[allow(non_upper_case_globals)]
pub const KSPROPSETID_CONNECTION: GUID = GUID {
    Data1: 0x1D58C920,
    Data2: 0xAC9B,
    Data3: 0x11CF,
    Data4: [0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00],
};

#[allow(non_upper_case_globals)]
pub static KSPROPSETID_GENERAL: GUID = GUID {
    Data1: 0x1464EDA5,
    Data2: 0x6A8F,
    Data3: 0x11D1,
    Data4: [0x9A, 0xA7, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub static KSPROPSETID_AUDIOEFFECTSDISCOVERY: GUID = GUID {
    Data1: 0xB49EEC73,
    Data2: 0xC88F,
    Data3: 0x40E1,
    Data4: [0x88, 0x48, 0xD3, 0xCE, 0x2C, 0x4B, 0x00, 0x51],
};

#[allow(non_upper_case_globals)]
pub static KSPROPSETID_AUDIOMODULE: GUID = GUID {
    Data1: 0xC034FDB0,
    Data2: 0xFF4C,
    Data3: 0x4788,
    Data4: [0xB3, 0xB6, 0xBF, 0x3E, 0x15, 0xCD, 0xC3, 0xE9],
};

pub const KSPROPERTY_GENERAL_COMPONENTID: u32 = 0;

#[allow(non_upper_case_globals)]
pub static KSPROPSETID_JACK: GUID = GUID {
    Data1: 0x4509F757,
    Data2: 0x2D46,
    Data3: 0x4637,
    Data4: [0x8E, 0x62, 0xCE, 0x7D, 0xB9, 0x44, 0xF5, 0x7B],
};

pub const KSPROPERTY_JACK_DESCRIPTION: u32 = 1;
pub const KSPROPERTY_JACK_DESCRIPTION2: u32 = 2;

#[allow(non_upper_case_globals)]
pub const KSPROPERTY_PIN_CATEGORY: u32 = 0;
pub const KSPROPERTY_PIN_NAME: u32 = 6;
pub const KSPROPERTY_PIN_PROPOSEDATAFORMAT: u32 = 12;
pub const KSPROPERTY_PIN_PROPOSEDATAFORMAT2: u32 = 17;

pub const KSPROPERTY_AUDIOEFFECTSDISCOVERY_EFFECTSLIST: u32 = 1;

pub const KSPROPERTY_AUDIOMODULE_DESCRIPTORS: u32 = 1;
pub const KSPROPERTY_AUDIOMODULE_COMMAND: u32 = 2;
pub const KSPROPERTY_AUDIOMODULE_NOTIFICATION_DEVICE_ID: u32 = 3;

pub const KSPROPERTY_SYSVAD_DEFAULTSTREAMEFFECTS: u32 = 1; // Sysvad-specific

pub const KSPROPERTY_TYPE_GET: u32 = 0x00000001;
pub const KSPROPERTY_TYPE_SET: u32 = 0x00000002;
pub const KSPROPERTY_TYPE_TOPOLOGY: u32 = 0x10000000;
pub const KSPROPERTY_TYPE_BASICSUPPORT: u32 = 0x00000200;

#[allow(non_upper_case_globals)]
pub static KSNODETYPE_SPEAKER: GUID = GUID {
    Data1: 0xDFF21CE1,
    Data2: 0xF70F,
    Data3: 0x11D0,
    Data4: [0xB9, 0x17, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub static KSNODETYPE_MICROPHONE: GUID = GUID {
    Data1: 0xDFF21BE1,
    Data2: 0xF70F,
    Data3: 0x11D0,
    Data4: [0xB9, 0x17, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

// ============================================================================
// Pin Indices & IDs
// ============================================================================

pub const PCFILTER_NODE: u32 = !0u32;

// KSPIN_DATAFLOW.
pub const KSPIN_DATAFLOW_IN: u32 = 1;
pub const KSPIN_DATAFLOW_OUT: u32 = 2;

// KSPIN_COMMUNICATION.
pub const KSPIN_COMMUNICATION_NONE: u32 = 0;
pub const KSPIN_COMMUNICATION_SINK: u32 = 1;
pub const KSPIN_COMMUNICATION_SOURCE: u32 = 2;
pub const KSPIN_COMMUNICATION_BOTH: u32 = 3;
pub const KSPIN_COMMUNICATION_BRIDGE: u32 = 4;

pub const KSPIN_WAVE_SINK: u32 = 0;
pub const KSPIN_WAVE_BRIDGE: u32 = 1;
pub const KSPIN_TOPO_BRIDGE: u32 = 0;
pub const KSPIN_TOPO_LINEOUT: u32 = 1;

// ============================================================================
// Topology Nodes & Audio Properties
// ============================================================================

#[allow(non_upper_case_globals)]
pub static KSNODETYPE_VOLUME: GUID = GUID {
    Data1: 0xDFF22003,
    Data2: 0xF70F,
    Data3: 0x11D0,
    Data4: [0xB9, 0x17, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub static KSNODETYPE_MUTE: GUID = GUID {
    Data1: 0x02F1A93E,
    Data2: 0x7E5F,
    Data3: 0x11D2,
    Data4: [0xA4, 0x4F, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96],
};

#[allow(non_upper_case_globals)]
pub static KSPROPSETID_AUDIO: GUID = GUID {
    Data1: 0x45FFAAA0,
    Data2: 0x6E1B,
    Data3: 0x11D0,
    Data4: [0xBC, 0xF2, 0x44, 0x45, 0x53, 0x54, 0x00, 0x00],
};

pub const KSPROPERTY_AUDIO_VOLUMELEVEL: u32 = 4;
pub const KSPROPERTY_AUDIO_MUTE: u32 = 5;

#[allow(non_upper_case_globals)]
pub static KSAUDFNAME_MASTER_VOLUME: GUID = GUID {
    Data1: 0x185FEDE0,
    Data2: 0x9905,
    Data3: 0x11D1,
    Data4: [0x95, 0xA9, 0x00, 0xC0, 0x4F, 0xB9, 0x25, 0xD3],
};

#[allow(non_upper_case_globals)]
pub static KSAUDFNAME_MASTER_MUTE: GUID = GUID {
    Data1: 0x185FEDE1,
    Data2: 0x9905,
    Data3: 0x11D1,
    Data4: [0x95, 0xA9, 0x00, 0xC0, 0x4F, 0xB9, 0x25, 0xD3],
};
