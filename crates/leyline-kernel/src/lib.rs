// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

#![no_std]

use wdk_sys::*;
use windows_drivers::nt_result::NtResult;

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    _driver_object: PDRIVER_OBJECT,
    _registry_path: PUNICODE_STRING,
) -> NTSTATUS {
    // TODO: Initialize WaveRT Miniport
    STATUS_SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
