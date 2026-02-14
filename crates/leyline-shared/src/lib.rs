// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

#![no_std]

/// GUID for the Leyline Audio Adapter.
/// {77B815C7-37B1-4A2D-A1A3-1A2B3C4D5E6F}
pub const LEYLINE_ADAPTER_GUID: [u8; 16] = [
    0xC7, 0x15, 0xB8, 0x77, 0xB1, 0x37, 0x2D, 0x4A, 0xA1, 0xA3, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E, 0x6F,
];

/// IOCTL code for setting buffer configuration from HSA.
pub const IOCTL_LEYLINE_SET_CONFIG: u32 = 0x80002000;

/// IOCTL code for getting driver status.
pub const IOCTL_LEYLINE_GET_STATUS: u32 = 0x80002004;
