// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

#![no_std]

pub mod buffer;
pub mod math;

// ============================================================================
// Driver Identity & Interface GUIDs
// ============================================================================
// This section defines the unique identifiers used by the Leyline driver
// for device identification and PnP interface registration.

/// GUID for the Leyline Audio Adapter.
/// {77B815C7-37B1-4A2D-A1A3-1A2B3C4D5E6F}
pub const LEYLINE_ADAPTER_GUID: [u8; 16] = [
    0xC7, 0x15, 0xB8, 0x77, 0xB1, 0x37, 0x2D, 0x4A, 0xA1, 0xA3, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F,
];

/// GUID for the Leyline Audio Processing Object (APO) CLSID.
/// {C8D3E4F5-B6A7-4A2D-A1A3-1A2B3C4D5E6F}
pub const LEYLINE_APO_CLSID: [u8; 16] = [
    0xF5, 0xE4, 0xD3, 0xC8, 0xA7, 0xB6, 0x2D, 0x4A, 0xA1, 0xA3, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F,
];

/// GUID for the Leyline Audio Interface.
/// {A1B2C3D4-E5F6-4A2D-B3C4-D5E6F7A8B9C0}
pub const LEYLINE_INTERFACE_GUID: [u8; 16] = [
    0xD4, 0xC3, 0xB2, 0xA1, 0xF6, 0xE5, 0x2D, 0x4A, 0xB3, 0xC4, 0xD5, 0xE6, 0xF7, 0xA8, 0xB9, 0xC0,
];

// ============================================================================
// Standard Windows Audio Categories
// ============================================================================
// These GUIDs correspond to standard KS/PortCls categories required for
// audio device enumeration and topology definition.

/// KSCATEGORY_AUDIO
/// {6994AD04-93EF-11D0-A3CC-00A0C9223196}
pub const KSCATEGORY_AUDIO: [u8; 16] = [
    0x04, 0xAD, 0x94, 0x69, 0xEF, 0x93, 0xD0, 0x11, 0xA3, 0xCC, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96,
];

/// KSCATEGORY_RENDER
/// {65E8773E-8F56-11D0-A3B9-00A0C9223196}
pub const KSCATEGORY_RENDER: [u8; 16] = [
    0x3E, 0x77, 0xE8, 0x65, 0x56, 0x8F, 0xD0, 0x11, 0xA3, 0xB9, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96,
];

/// KSCATEGORY_CAPTURE
/// {65E8773D-8F56-11D0-A3B9-00A0C9223196}
pub const KSCATEGORY_CAPTURE: [u8; 16] = [
    0x3D, 0x77, 0xE8, 0x65, 0x56, 0x8F, 0xD0, 0x11, 0xA3, 0xB9, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96,
];

/// KSCATEGORY_REALTIME
/// {EB115AD5-9118-4FA0-BD83-ED352215DF43}
pub const KSCATEGORY_REALTIME: [u8; 16] = [
    0xD5, 0x5A, 0x11, 0xEB, 0x18, 0x91, 0xA0, 0x4F, 0xBD, 0x83, 0xED, 0x35, 0x22, 0x15, 0xDF, 0x43,
];

// ============================================================================
// Control Codes (IOCTLs)
// ============================================================================
// IOCTL codes for communication between the Hardware Support App (HSA)
// and the kernel-mode driver.

// Helper for defining IOCTLs (CTL_CODE macro logic)
const fn ctl_code(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    (device_type << 16) | (access << 14) | (function << 2) | method
}

const FILE_DEVICE_UNKNOWN: u32 = 0x00000022;
const METHOD_BUFFERED: u32 = 0;
const FILE_ANY_ACCESS: u32 = 0;

/// Shared parameters between HSA, APO, and Driver.
#[repr(C)]
pub struct SharedParameters {
    pub master_gain_bits: u32, // IEEE754 float bits
    pub peak_l_bits: u32,      // IEEE754 float bits
    pub peak_r_bits: u32,      // IEEE754 float bits
    pub qpc_frequency: i64,
    pub render_start_qpc: i64,
    pub capture_start_qpc: i64,
    pub buffer_size: u32,
    pub byte_rate: u32,
}

/// IOCTL code for setting buffer configuration from HSA.
pub const IOCTL_LEYLINE_SET_CONFIG: u32 =
    ctl_code(FILE_DEVICE_UNKNOWN, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS);

/// IOCTL code for getting driver status.
pub const IOCTL_LEYLINE_GET_STATUS: u32 =
    ctl_code(FILE_DEVICE_UNKNOWN, 0x801, METHOD_BUFFERED, FILE_ANY_ACCESS);

/// IOCTL code for mapping the shared audio buffer to user-space.
pub const IOCTL_LEYLINE_MAP_BUFFER: u32 =
    ctl_code(FILE_DEVICE_UNKNOWN, 0x802, METHOD_BUFFERED, FILE_ANY_ACCESS);

/// IOCTL code for mapping the shared parameter block to user-space.
pub const IOCTL_LEYLINE_MAP_PARAMS: u32 =
    ctl_code(FILE_DEVICE_UNKNOWN, 0x803, METHOD_BUFFERED, FILE_ANY_ACCESS);
