// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ACX CIRCUIT CREATION
// Logic for defining and adding ACX endpoints (render and capture).
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use wdk_sys::ntddk::*;
use wdk_sys::*;

#[allow(unused_variables)]
pub unsafe fn create_render_circuit(device: WDFDEVICE) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: Creating Render Circuit\n".as_ptr());
    // TODO
    STATUS_SUCCESS
}

#[allow(unused_variables)]
pub unsafe fn create_capture_circuit(device: WDFDEVICE) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: Creating Capture Circuit\n".as_ptr());
    // TODO
    STATUS_SUCCESS
}
