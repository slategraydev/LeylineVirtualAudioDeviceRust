// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

#![no_std]

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
pub const KSCATEGORY_AUDIO: [u8; 16] = [
    0x98, 0x33, 0x22, 0x69, 0x6C, 0x30, 0x11, 0xCF, 0xB5, 0xCA, 0x00, 0x80, 0x5F, 0x48, 0xA1, 0x92,
];

/// KSCATEGORY_RENDER
pub const KSCATEGORY_RENDER: [u8; 16] = [
    0x33, 0xCC, 0x71, 0x65, 0x35, 0x80, 0x11, 0xD0, 0xA7, 0x08, 0x00, 0xA0, 0xC9, 0x03, 0x49, 0x02,
];

/// KSCATEGORY_CAPTURE
pub const KSCATEGORY_CAPTURE: [u8; 16] = [
    0x34, 0xCC, 0x71, 0x65, 0x35, 0x80, 0x11, 0xD0, 0xA7, 0x08, 0x00, 0xA0, 0xC9, 0x03, 0x49, 0x02,
];

// ============================================================================
// Control Codes (IOCTLs)
// ============================================================================
// IOCTL codes for communication between the Hardware Support App (HSA)
// and the kernel-mode driver.

/// Shared parameters between HSA, APO, and Driver.
#[repr(C)]
pub struct SharedParameters {
    pub master_gain: f32,
    pub peak_l: f32,
    pub peak_r: f32,
}

/// IOCTL code for setting buffer configuration from HSA.
pub const IOCTL_LEYLINE_SET_CONFIG: u32 = 0x80002000;

/// IOCTL code for getting driver status.
pub const IOCTL_LEYLINE_GET_STATUS: u32 = 0x80002004;

/// IOCTL code for mapping the shared audio buffer to user-space.
pub const IOCTL_LEYLINE_MAP_BUFFER: u32 = 0x80002008;

/// IOCTL code for mapping the shared parameter block to user-space.
pub const IOCTL_LEYLINE_MAP_PARAMS: u32 = 0x8000200C;
