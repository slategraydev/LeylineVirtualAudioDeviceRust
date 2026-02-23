// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

// First std/core/alloc.
use alloc::boxed::Box;
use core::mem::size_of;
use core::ptr::null_mut;

// Second, external crates.
use wdk_sys::ntddk::*;
use wdk_sys::*;

// Then current crate.
use crate::constants::*;
use crate::dispatch::*;
use crate::stream::MiniportWaveRTStream;
use crate::topology::MiniportTopologyCom;
use crate::vtables::*;
use crate::wavert::MiniportWaveRTCom;
use crate::{PcAddAdapterDevice, PcNewPort, PcRegisterPhysicalConnection, PcRegisterSubdevice};

const _POOL_TAG: u32 = u32::from_be_bytes(*b"LLAD");
const PORT_CLASS_DEVICE_EXTENSION_SIZE: usize = 64 * size_of::<usize>();

#[repr(C)]
pub struct DeviceExtension {
    pub control_device_object: *mut DEVICE_OBJECT,
    pub shared_params: *mut leyline_shared::SharedParameters,
    pub shared_params_mdl: PMDL,
    pub shared_params_user_mapping: *mut u8,
    pub loopback_mdl: PMDL,
    pub loopback_buffer: *mut u8,
    pub loopback_size: usize,
    pub user_mapping: *mut u8,
    pub render_miniport: *mut MiniportWaveRTCom,
    pub capture_miniport: *mut MiniportWaveRTCom,
    pub render_topo_miniport: *mut MiniportTopologyCom,
    pub capture_topo_miniport: *mut MiniportTopologyCom,
}

#[repr(C)]
pub struct MiniportWaveRTStreamCom {
    pub vtable: *const IMiniportWaveRTStreamVTable,
    pub stream: *mut MiniportWaveRTStream,
    pub ref_count: u32,
}

#[link_section = ".rdata"]
pub static STREAM_VTABLE: IMiniportWaveRTStreamVTable = IMiniportWaveRTStreamVTable {
    base: IUnknownVTable {
        QueryInterface: crate::stream_query_interface,
        AddRef: crate::stream_add_ref,
        Release: crate::stream_release,
    },
    SetFormat: crate::stream_set_format,
    SetState: crate::stream_set_state,
    GetPosition: crate::stream_get_position,
    AllocateAudioBuffer: crate::stream_allocate_audio_buffer,
    FreeAudioBuffer: crate::stream_free_audio_buffer,
    GetHWLatency: crate::stream_get_hw_latency,
    GetPositionRegister: crate::stream_get_position_register,
    GetClockRegister: crate::stream_get_clock_register,
};

impl MiniportWaveRTStreamCom {
    pub fn new(stream: *mut MiniportWaveRTStream) -> Box<Self> {
        Box::new(Self {
            vtable: &STREAM_VTABLE,
            stream,
            ref_count: 1,
        })
    }
}

/// Retrieves the Leyline device extension from a PortCls device object.
///
/// # Safety
/// The provided device object must be a valid PortCls-initialized device object.
#[inline(always)]
pub unsafe fn get_device_extension(device_object: PDEVICE_OBJECT) -> *mut DeviceExtension {
    let base = (*device_object).DeviceExtension as *mut u8;
    base.add(PORT_CLASS_DEVICE_EXTENSION_SIZE) as *mut DeviceExtension
}

// Session #42: Explicit audio device interface registration
// Required because INF AddInterface is not being processed for virtual audio drivers
// --- REMOVED Session #46 (Aligned with PortCls/INF architecture) ---

/// AddDevice callback for PortCls initialization.
///
/// # Safety
/// Standard kernel AddDevice callback. Parameters must be valid pointers provided by the OS.
#[allow(non_snake_case)]
pub unsafe extern "C" fn AddDevice(
    driver_object: PDRIVER_OBJECT,
    physical_device_object: PDEVICE_OBJECT,
) -> NTSTATUS {
    DbgPrint(
        c"Leyline: AddDevice (PDO: %p)\n".as_ptr(),
        physical_device_object,
    );

    // Diagnostic: Check the Hardware ID of the PDO
    let mut length: u32 = 0;
    let _ = IoGetDeviceProperty(
        physical_device_object,
        DEVICE_REGISTRY_PROPERTY::DevicePropertyHardwareID,
        0,
        core::ptr::null_mut(),
        &mut length,
    );
    if length > 0 {
        let mut buffer = alloc::vec![0u8; length as usize];
        let status = IoGetDeviceProperty(
            physical_device_object,
            DEVICE_REGISTRY_PROPERTY::DevicePropertyHardwareID,
            length,
            buffer.as_mut_ptr() as PVOID,
            &mut length,
        );
        if status == STATUS_SUCCESS {
            // Hardware IDs are multi-sz (null separated).
            DbgPrint(
                c"Leyline: PDO Hardware ID: %ls\n".as_ptr(),
                buffer.as_ptr() as *const u16,
            );
        }
    }

    let total_extension_size =
        (PORT_CLASS_DEVICE_EXTENSION_SIZE + size_of::<DeviceExtension>()) as u32;

    PcAddAdapterDevice(
        driver_object,
        physical_device_object,
        Some(StartDevice),
        10,
        total_extension_size,
    )
}

/// StartDevice callback for hardware initialization.
///
/// # Safety
/// Standard kernel StartDevice callback. Parameters must be valid pointers provided by the OS.
#[allow(non_snake_case)]
pub unsafe extern "C" fn StartDevice(
    device_object: PDEVICE_OBJECT,
    _irp: PIRP,
    resource_list: PVOID,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let dev_ext = get_device_extension(device_object);
    DbgPrint(c"Leyline: StartDevice (FDO: %p)\n".as_ptr(), device_object);

    // --- WaveRender Registration ---
    DbgPrint(c"Leyline: Registering WaveRender Port\n".as_ptr());
    let mut render_port: *mut u8 = null_mut();
    status = PcNewPort(&mut render_port, &CLSID_PortWaveRT);
    if status != STATUS_SUCCESS {
        return status;
    }

    let render_miniport_com = MiniportWaveRTCom::new(false, dev_ext);
    let render_miniport_ptr = Box::into_raw(render_miniport_com) as *mut u8;
    (*dev_ext).render_miniport = render_miniport_ptr as *mut MiniportWaveRTCom;

    type PortInitFn = unsafe extern "system" fn(
        this: *mut u8,
        device_object: PDEVICE_OBJECT,
        irp: PIRP,
        miniport: *mut u8,
        unknown_adapter: PVOID,
        resource_list: PVOID,
    ) -> NTSTATUS;

    let vtable = *(render_port as *const *const *const u8);
    let init_ptr = *vtable.add(3);
    let init_fn: PortInitFn = core::mem::transmute(init_ptr);

    status = init_fn(
        render_port,
        device_object,
        _irp,
        render_miniport_ptr,
        null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        return status;
    }

    let mut wave_render_name_buffer = [0u16; 11];
    let wave_render_name_str = "WaveRender";
    for (i, c) in wave_render_name_str.encode_utf16().enumerate() {
        wave_render_name_buffer[i] = c;
    }
    status = PcRegisterSubdevice(device_object, wave_render_name_buffer.as_ptr(), render_port);
    if status != STATUS_SUCCESS {
        return status;
    }
    DbgPrint(c"Leyline: WaveRender Subdevice Registered\n".as_ptr());

    // --- WaveCapture Registration ---
    DbgPrint(c"Leyline: Registering WaveCapture Port\n".as_ptr());
    let mut capture_port: *mut u8 = null_mut();
    status = PcNewPort(&mut capture_port, &CLSID_PortWaveRT);
    if status != STATUS_SUCCESS {
        return status;
    }

    let capture_miniport_com = MiniportWaveRTCom::new(true, dev_ext);
    let capture_miniport_ptr = Box::into_raw(capture_miniport_com) as *mut u8;
    (*dev_ext).capture_miniport = capture_miniport_ptr as *mut MiniportWaveRTCom;

    let vtable = *(capture_port as *const *const *const u8);
    let init_ptr = *vtable.add(3);
    let capture_init_fn: PortInitFn = core::mem::transmute(init_ptr);

    status = capture_init_fn(
        capture_port,
        device_object,
        _irp,
        capture_miniport_ptr,
        null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        return status;
    }

    let mut wave_capture_name_buffer = [0u16; 12];
    let wave_capture_name_str = "WaveCapture";
    for (i, c) in wave_capture_name_str.encode_utf16().enumerate() {
        wave_capture_name_buffer[i] = c;
    }
    status = PcRegisterSubdevice(
        device_object,
        wave_capture_name_buffer.as_ptr(),
        capture_port,
    );
    if status != STATUS_SUCCESS {
        DbgPrint(c"Leyline: PcRegisterSubdevice(WaveCapture) Failed\n".as_ptr());
        return status;
    }
    DbgPrint(c"Leyline: WaveCapture Subdevice Registered\n".as_ptr());

    // --- Topology Registration (Render Only for Diagnosis) ---
    DbgPrint(c"Leyline: Registering TopologyRender Port\n".as_ptr());
    DbgPrint(c"Leyline: About to call PcNewPort with CLSID_PortTopology\n".as_ptr());
    let mut render_topo_port: *mut u8 = null_mut();
    status = PcNewPort(&mut render_topo_port, &CLSID_PortTopology);
    if status != STATUS_SUCCESS {
        DbgPrint(c"Leyline: PcNewPort(TopologyRender) FAILED\n".as_ptr());
        // Status 0xC00002B9 = STATUS_REQUEST_NOT_ACCEPTED
        // This typically means PortCls rejected the creation request
        if status == 0xC00002B9u32 as i32 {
            DbgPrint(c"Leyline: ERROR - STATUS_REQUEST_NOT_ACCEPTED (0xC00002B9)\n".as_ptr());
            DbgPrint(c"Leyline: Possible causes:\n".as_ptr());
            DbgPrint(c"Leyline:   - Invalid/malformed miniport descriptor\n".as_ptr());
            DbgPrint(c"Leyline:   - Missing interface support in miniport\n".as_ptr());
            DbgPrint(c"Leyline:   - PortCls unable to initialize topology port\n".as_ptr());
        }
        return status;
    }
    DbgPrint(c"Leyline: PcNewPort(TopologyRender) SUCCESS\n".as_ptr());

    let topo_miniport_com = MiniportTopologyCom::new(false); // false = Render
    let topo_miniport_ptr = Box::into_raw(topo_miniport_com) as *mut u8;
    (*dev_ext).render_topo_miniport = topo_miniport_ptr as *mut MiniportTopologyCom;

    let vtable = *(render_topo_port as *const *const *const u8);
    // IPortTopology inherits IPort. IPortTopology inherits IUnknown. Init is index 3.
    let init_ptr = *vtable.add(3);
    let init_fn: PortInitFn = core::mem::transmute(init_ptr);

    DbgPrint(c"Leyline: Calling TopologyRender::Init\n".as_ptr());
    DbgPrint(c"Leyline: Init function pointer acquired from vtable[3]\n".as_ptr());

    // Validate miniport pointer before passing
    if topo_miniport_ptr.is_null() {
        DbgPrint(c"Leyline: ERROR - topo_miniport_ptr is NULL!\n".as_ptr());
        return STATUS_INVALID_PARAMETER;
    }
    DbgPrint(c"Leyline: Miniport pointer is valid\n".as_ptr());

    status = init_fn(
        render_topo_port,
        device_object,
        _irp,
        topo_miniport_ptr,
        null_mut(),
        null_mut(), // Topology doesn't need hardware resources
    );
    if status != STATUS_SUCCESS {
        DbgPrint(c"Leyline: TopologyRender::Init FAILED\n".as_ptr());
        if status == 0xC00002B9u32 as i32 {
            DbgPrint(c"Leyline: ERROR - STATUS_REQUEST_NOT_ACCEPTED during Init\n".as_ptr());
            DbgPrint(c"Leyline: The miniport rejected the initialization request\n".as_ptr());
            DbgPrint(c"Leyline: Check DbgPrint output from topology miniport above\n".as_ptr());
        }
        return status;
    }
    DbgPrint(c"Leyline: TopologyRender::Init SUCCESS\n".as_ptr());

    let mut topo_render_name_buffer = [0u16; 15];
    let topo_render_name_str = "TopologyRender";
    for (i, c) in topo_render_name_str.encode_utf16().enumerate() {
        topo_render_name_buffer[i] = c;
    }
    DbgPrint(c"Leyline: Registering TopologyRender Subdevice\n".as_ptr());
    status = PcRegisterSubdevice(
        device_object,
        topo_render_name_buffer.as_ptr(),
        render_topo_port,
    );
    if status != STATUS_SUCCESS {
        DbgPrint(c"Leyline: PcRegisterSubdevice(TopologyRender) Failed\n".as_ptr());
        return status;
    }
    DbgPrint(c"Leyline: TopologyRender Subdevice Registered\n".as_ptr());

    // --- Physical Connection: WaveRender (Pin 1) -> TopologyRender (Pin 0) ---
    DbgPrint(c"Leyline: Physical Connection (WaveRender Pin 1 -> TopologyRender Pin 0)\n".as_ptr());
    status = PcRegisterPhysicalConnection(
        device_object,
        render_port as *mut _,
        1,
        render_topo_port as *mut _,
        0,
    );
    if status != STATUS_SUCCESS {
        DbgPrint(
            c"Leyline: ERROR - Physical Connection (Wave->Topo) FAILED: 0x%x\n".as_ptr(),
            status,
        );
        return status;
    }
    DbgPrint(c"Leyline: Physical Connection (Wave->Topo) SUCCESS\n".as_ptr());

    // --- Topology Capture Registration ---
    DbgPrint(c"Leyline: Registering TopologyCapture Port\n".as_ptr());
    let mut capture_topo_port: *mut u8 = null_mut();
    status = PcNewPort(&mut capture_topo_port, &CLSID_PortTopology);
    if status != STATUS_SUCCESS {
        DbgPrint(c"Leyline: PcNewPort(TopologyCapture) FAILED\n".as_ptr());
        return status;
    }
    DbgPrint(c"Leyline: PcNewPort(TopologyCapture) SUCCESS\n".as_ptr());

    let capture_topo_miniport_com = MiniportTopologyCom::new(true); // true = Capture
    let capture_topo_miniport_ptr = Box::into_raw(capture_topo_miniport_com) as *mut u8;
    (*dev_ext).capture_topo_miniport = capture_topo_miniport_ptr as *mut MiniportTopologyCom;

    let vtable = *(capture_topo_port as *const *const *const u8);
    let init_ptr = *vtable.add(3);
    let capture_topo_init_fn: PortInitFn = core::mem::transmute(init_ptr);

    DbgPrint(c"Leyline: Calling TopologyCapture::Init\n".as_ptr());
    status = capture_topo_init_fn(
        capture_topo_port,
        device_object,
        _irp,
        capture_topo_miniport_ptr,
        null_mut(),
        null_mut(),
    );
    if status != STATUS_SUCCESS {
        DbgPrint(c"Leyline: TopologyCapture::Init FAILED\n".as_ptr());
        return status;
    }
    DbgPrint(c"Leyline: TopologyCapture::Init SUCCESS\n".as_ptr());

    let mut topo_capture_name_buffer = [0u16; 16];
    let topo_capture_name_str = "TopologyCapture";
    for (i, c) in topo_capture_name_str.encode_utf16().enumerate() {
        topo_capture_name_buffer[i] = c;
    }
    DbgPrint(c"Leyline: Registering TopologyCapture Subdevice\n".as_ptr());
    status = PcRegisterSubdevice(
        device_object,
        topo_capture_name_buffer.as_ptr(),
        capture_topo_port,
    );
    if status != STATUS_SUCCESS {
        DbgPrint(c"Leyline: PcRegisterSubdevice(TopologyCapture) Failed\n".as_ptr());
        return status;
    }
    DbgPrint(c"Leyline: TopologyCapture Subdevice Registered\n".as_ptr());

    // --- Physical Connection: TopologyCapture (Pin 1) -> WaveCapture (Pin 1) ---
    DbgPrint(
        c"Leyline: Physical Connection (TopologyCapture Pin 1 -> WaveCapture Pin 1)\n".as_ptr(),
    );
    status = PcRegisterPhysicalConnection(
        device_object,
        capture_topo_port as *mut _,
        1, // Topo bridge pin
        capture_port as *mut _,
        1, // Wave bridge pin
    );
    if status != STATUS_SUCCESS {
        DbgPrint(
            c"Leyline: ERROR - Physical Connection (Topo->Wave) FAILED: 0x%x\n".as_ptr(),
            status,
        );
        return status;
    }
    DbgPrint(c"Leyline: Physical Connection (Topo->Wave) SUCCESS\n".as_ptr());

    // --- CDO Creation (Moved to end to prevent PnP interference) ---
    crate::FUNCTIONAL_DEVICE_OBJECT = device_object;
    if CONTROL_DEVICE_OBJECT.is_null() {
        let mut device_name_str = [0u16; 20];
        let name_prefix = r"\Device\LeylineAudio";
        for (i, c) in name_prefix.encode_utf16().enumerate() {
            device_name_str[i] = c;
        }
        let mut device_name = UNICODE_STRING {
            Length: (name_prefix.len() * 2) as u16,
            MaximumLength: (device_name_str.len() * 2) as u16,
            Buffer: device_name_str.as_mut_ptr(),
        };

        status = IoCreateDevice(
            (*device_object).DriverObject,
            size_of::<usize>() as u32,
            &mut device_name,
            FILE_DEVICE_UNKNOWN,
            0,
            0,
            &raw mut CONTROL_DEVICE_OBJECT,
        );
        if status == STATUS_SUCCESS {
            let mut link_name_str = [0u16; 25];
            let link_prefix = r"\DosDevices\LeylineAudio";
            for (i, c) in link_prefix.encode_utf16().enumerate() {
                link_name_str[i] = c;
            }
            let mut link_name = UNICODE_STRING {
                Length: (link_prefix.len() * 2) as u16,
                MaximumLength: (link_name_str.len() * 2) as u16,
                Buffer: link_name_str.as_mut_ptr(),
            };
            let _ = IoCreateSymbolicLink(&mut link_name, &mut device_name);
            DbgPrint(c"Leyline: CDO Ready\n".as_ptr());
        } else if status == 0xC0000035u32 as i32 {
            // STATUS_OBJECT_NAME_COLLISION - This is fine, the CDO might already exist
            // from a previous start attempt or a different FDO instance in the same stack.
            DbgPrint(c"Leyline: CDO already exists (0xc0000035), skipping creation\n".as_ptr());
            status = STATUS_SUCCESS;
        }
    } else {
        // CDO already non-null in our variable
        status = STATUS_SUCCESS;
    }

    if status == STATUS_SUCCESS {
        DbgPrint(c"Leyline: ==================================================\n".as_ptr());
        DbgPrint(c"Leyline: StartDevice COMPLETED SUCCESSFULLY v1.0.150 (REBUILT)\n".as_ptr());
        DbgPrint(c"Leyline: Registered Subdevices:\n".as_ptr());
        DbgPrint(c"Leyline:   - WaveRender (Output)\n".as_ptr());
        DbgPrint(c"Leyline:   - WaveCapture (Input)\n".as_ptr());
        DbgPrint(c"Leyline:   - TopologyRender\n".as_ptr());
        DbgPrint(c"Leyline:   - TopologyCapture\n".as_ptr());
        DbgPrint(c"Leyline: Physical Connections:\n".as_ptr());
        DbgPrint(c"Leyline:   - WaveRender -> TopologyRender\n".as_ptr());
        DbgPrint(c"Leyline:   - TopologyCapture Pin 1 -> WaveCapture Pin 1\n".as_ptr());
        DbgPrint(c"Leyline: ==================================================\n".as_ptr());
    } else {
        DbgPrint(
            c"Leyline: StartDevice FAILED with status: 0x%x\n".as_ptr(),
            status,
        );
    }
    status
}
