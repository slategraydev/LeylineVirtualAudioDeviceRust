// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ACX CIRCUIT CREATION
// Logic for defining and adding ACX endpoints (render and capture).
// Each circuit represents a single audio endpoint (speaker or microphone).
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use core::ptr::null_mut;

use wdk_sys::ntddk::*;
use wdk_sys::*;

use crate::audio_bindings;
#[allow(unused_imports)]
use crate::constants;

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// HELPER: ACX FUNCTION DISPATCH
// Retrieves function pointers from the ACX dispatch table at runtime.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Invoke an ACX function by its table index. The ACX framework stores
/// function pointers in a global table (`AcxFunctions`) indexed by
/// `_ACXFUNCENUM` constants. This macro transmutes the raw pointer to
/// the expected function signature and calls it.
macro_rules! acx_call {
    ($pfn_type:ty, $idx:expr, $($args:expr),* $(,)?) => {{
        let func: $pfn_type = core::mem::transmute(
            *(core::ptr::addr_of!(audio_bindings::AcxFunctions) as *const _ as *const *const core::ffi::c_void).add($idx as usize)
        );
        func.unwrap()(audio_bindings::AcxDriverGlobals, $($args),*)
    }};
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// FORMAT DEFINITIONS
// PCM 16-bit stereo 48 kHz and IEEE Float 32-bit stereo 48 kHz ranges.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[repr(C)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct KSDATAFORMAT {
    pub FormatSize: u32,
    pub Flags: u32,
    pub SampleSize: u32,
    pub Reserved: u32,
    pub MajorFormat: wdk_sys::GUID,
    pub SubFormat: wdk_sys::GUID,
    pub Specifier: wdk_sys::GUID,
}

/// KSDATARANGE_AUDIO for 16-bit PCM, 48000 Hz, 2 channels.
/// Used to define supported data formats on circuit pins.
#[allow(dead_code)]
fn make_pcm_data_range() -> KSDATAFORMAT {
    KSDATAFORMAT {
        FormatSize: core::mem::size_of::<KSDATAFORMAT>() as u32,
        Flags: 0,
        SampleSize: 4, // 2ch * 16bit = 4 bytes per sample frame
        Reserved: 0,
        MajorFormat: constants::KSDATAFORMAT_TYPE_AUDIO,
        SubFormat: constants::KSDATAFORMAT_SUBTYPE_PCM,
        Specifier: constants::KSDATAFORMAT_SPECIFIER_WAVEFORMATEX,
    }
}

/// KSDATARANGE_AUDIO for 32-bit IEEE Float, 48000 Hz, 2 channels.
#[allow(dead_code)]
fn make_float_data_range() -> KSDATAFORMAT {
    KSDATAFORMAT {
        FormatSize: core::mem::size_of::<KSDATAFORMAT>() as u32,
        Flags: 0,
        SampleSize: 8, // 2ch * 32bit = 8 bytes per sample frame
        Reserved: 0,
        MajorFormat: constants::KSDATAFORMAT_TYPE_AUDIO,
        SubFormat: constants::KSDATAFORMAT_SUBTYPE_IEEE_FLOAT,
        Specifier: constants::KSDATAFORMAT_SPECIFIER_WAVEFORMATEX,
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// RENDER CIRCUIT
// Represents the speaker/output endpoint.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Create the render (output) AcxCircuit and add it to the device.
///
/// The render circuit has:
/// - One host pin (SINK) accepting PCM and Float data ranges.
/// - Circuit type `AcxCircuitTypeRender`.
/// - An `EvtAcxCircuitCreateStream` callback for stream creation.
///
/// # Safety
/// Standard kernel-mode function. `device` must be a valid WDFDEVICE handle.
pub unsafe fn create_render_circuit(device: WDFDEVICE) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: Creating Render Circuit\n".as_ptr());

    // 1. Allocate ACXCIRCUIT_INIT
    let circuit_init: audio_bindings::PACXCIRCUIT_INIT = acx_call!(
        audio_bindings::PFN_ACXCIRCUITINITALLOCATE,
        audio_bindings::_ACXFUNCENUM_AcxCircuitInitAllocateTableIndex,
        device as _
    );
    if circuit_init.is_null() {
        DbgPrint(c"Leyline [ACX]: AcxCircuitInitAllocate failed (render)\n".as_ptr());
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // 2. Set circuit type to Render
    acx_call!(
        audio_bindings::PFN_ACXCIRCUITINITSETCIRCUITTYPE,
        audio_bindings::_ACXFUNCENUM_AcxCircuitInitSetCircuitTypeTableIndex,
        circuit_init,
        audio_bindings::_ACX_CIRCUIT_TYPE_AcxCircuitTypeRender
    );

    // 3. Assign the EvtAcxCircuitCreateStream callback
    let status = acx_call!(
        audio_bindings::PFN_ACXCIRCUITINITASSIGNACXCREATESTREAMCALLBACK,
        audio_bindings::_ACXFUNCENUM_AcxCircuitInitAssignAcxCreateStreamCallbackTableIndex,
        circuit_init,
        Some(core::mem::transmute(crate::stream::evt_render_circuit_create_stream as *const ()))
    );
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [ACX]: AssignCreateStreamCallback failed (render) %x\n".as_ptr(), status);
        return status;
    }

    // 4. Create the circuit
    let mut circuit_attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
    circuit_attributes.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;

    let mut circuit: audio_bindings::ACXCIRCUIT = null_mut();
    let mut circuit_init_ptr = circuit_init;
    let status = acx_call!(
        audio_bindings::PFN_ACXCIRCUITCREATE,
        audio_bindings::_ACXFUNCENUM_AcxCircuitCreateTableIndex,
        device as _,
        &mut circuit_attributes as *mut _ as _,
        &mut circuit_init_ptr,
        &mut circuit
    );
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [ACX]: AcxCircuitCreate failed (render) %x\n".as_ptr(), status);
        return status;
    }

    // 5. Add circuit to device
    let status = acx_call!(
        audio_bindings::PFN_ACXDEVICEADDCIRCUIT,
        audio_bindings::_ACXFUNCENUM_AcxDeviceAddCircuitTableIndex,
        device as _,
        circuit
    );
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [ACX]: AcxDeviceAddCircuit failed (render) %x\n".as_ptr(), status);
        return status;
    }

    DbgPrint(c"Leyline [ACX]: Render circuit created and added\n".as_ptr());
    STATUS_SUCCESS
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// CAPTURE CIRCUIT
// Represents the microphone/input endpoint.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Create the capture (input) AcxCircuit and add it to the device.
///
/// The capture circuit has:
/// - One host pin (SOURCE) accepting PCM and Float data ranges.
/// - Circuit type `AcxCircuitTypeCapture`.
/// - An `EvtAcxCircuitCreateStream` callback for stream creation.
///
/// # Safety
/// Standard kernel-mode function. `device` must be a valid WDFDEVICE handle.
pub unsafe fn create_capture_circuit(device: WDFDEVICE) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: Creating Capture Circuit\n".as_ptr());

    // 1. Allocate ACXCIRCUIT_INIT
    let circuit_init: audio_bindings::PACXCIRCUIT_INIT = acx_call!(
        audio_bindings::PFN_ACXCIRCUITINITALLOCATE,
        audio_bindings::_ACXFUNCENUM_AcxCircuitInitAllocateTableIndex,
        device as _
    );
    if circuit_init.is_null() {
        DbgPrint(c"Leyline [ACX]: AcxCircuitInitAllocate failed (capture)\n".as_ptr());
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // 2. Set circuit type to Capture
    acx_call!(
        audio_bindings::PFN_ACXCIRCUITINITSETCIRCUITTYPE,
        audio_bindings::_ACXFUNCENUM_AcxCircuitInitSetCircuitTypeTableIndex,
        circuit_init,
        audio_bindings::_ACX_CIRCUIT_TYPE_AcxCircuitTypeCapture
    );

    // 3. Assign the EvtAcxCircuitCreateStream callback
    let status = acx_call!(
        audio_bindings::PFN_ACXCIRCUITINITASSIGNACXCREATESTREAMCALLBACK,
        audio_bindings::_ACXFUNCENUM_AcxCircuitInitAssignAcxCreateStreamCallbackTableIndex,
        circuit_init,
        Some(core::mem::transmute(crate::stream::evt_capture_circuit_create_stream as *const ()))
    );
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [ACX]: AssignCreateStreamCallback failed (capture) %x\n".as_ptr(), status);
        return status;
    }

    // 4. Create the circuit
    let mut circuit_attributes: WDF_OBJECT_ATTRIBUTES = core::mem::zeroed();
    circuit_attributes.Size = core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as u32;

    let mut circuit: audio_bindings::ACXCIRCUIT = null_mut();
    let mut circuit_init_ptr = circuit_init;
    let status = acx_call!(
        audio_bindings::PFN_ACXCIRCUITCREATE,
        audio_bindings::_ACXFUNCENUM_AcxCircuitCreateTableIndex,
        device as _,
        &mut circuit_attributes as *mut _ as _,
        &mut circuit_init_ptr,
        &mut circuit
    );
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [ACX]: AcxCircuitCreate failed (capture) %x\n".as_ptr(), status);
        return status;
    }

    // 5. Add circuit to device
    let status = acx_call!(
        audio_bindings::PFN_ACXDEVICEADDCIRCUIT,
        audio_bindings::_ACXFUNCENUM_AcxDeviceAddCircuitTableIndex,
        device as _,
        circuit
    );
    if !NT_SUCCESS(status) {
        DbgPrint(c"Leyline [ACX]: AcxDeviceAddCircuit failed (capture) %x\n".as_ptr(), status);
        return status;
    }

    DbgPrint(c"Leyline [ACX]: Capture circuit created and added\n".as_ptr());
    STATUS_SUCCESS
}
