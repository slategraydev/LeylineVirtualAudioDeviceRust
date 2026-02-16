// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

use crate::constants::*;
use crate::dispatch::*;
use crate::stream::MiniportWaveRTStream;
use crate::topology::MiniportTopologyCom;
use crate::vtables::*;
use crate::wavert::MiniportWaveRTCom;
use alloc::boxed::Box;
use wdk_sys::ntddk::*;
use wdk_sys::*;

const _POOL_TAG: u32 = u32::from_be_bytes(*b"LLAD");
const PORT_CLASS_DEVICE_EXTENSION_SIZE: usize = 64 * core::mem::size_of::<usize>();

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

impl MiniportWaveRTStreamCom {
    pub fn new(stream: *mut MiniportWaveRTStream) -> Box<Self> {
        Box::new(Self {
            vtable: &STREAM_VTABLE,
            stream,
            ref_count: 1,
        })
    }
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

#[inline(always)]
pub unsafe fn get_device_extension(device_object: PDEVICE_OBJECT) -> *mut DeviceExtension {
    let base = (*device_object).DeviceExtension as *mut u8;
    base.add(PORT_CLASS_DEVICE_EXTENSION_SIZE) as *mut DeviceExtension
}

#[link(name = "portcls")]
extern "C" {
    pub fn PcAddAdapterDevice(
        DriverObject: PDRIVER_OBJECT,
        PhysicalDeviceObject: PDEVICE_OBJECT,
        StartDevice: Option<unsafe extern "C" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS>,
        MaxObjects: u32,
        DeviceExtensionSize: u32,
    ) -> NTSTATUS;

    pub fn PcNewPort(OutPort: *mut *mut u8, ClassId: *const GUID) -> NTSTATUS;

    pub fn PcRegisterSubdevice(
        DeviceObject: PDEVICE_OBJECT,
        Name: *const u16,
        Unknown: *mut u8,
    ) -> NTSTATUS;
}

pub unsafe extern "C" fn AddDevice(
    driver_object: PDRIVER_OBJECT,
    physical_device_object: PDEVICE_OBJECT,
) -> NTSTATUS {
    DbgPrint("Leyline: AddDevice\n\0".as_ptr() as *const i8);

    let total_extension_size =
        (PORT_CLASS_DEVICE_EXTENSION_SIZE + core::mem::size_of::<DeviceExtension>()) as u32;

    PcAddAdapterDevice(
        driver_object,
        physical_device_object,
        Some(StartDevice),
        10,
        total_extension_size,
    )
}

pub unsafe extern "C" fn StartDevice(
    device_object: PDEVICE_OBJECT,
    _irp: PIRP,
    resource_list: PVOID,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let dev_ext = get_device_extension(device_object);
    DbgPrint("Leyline: StartDevice\n\0".as_ptr() as *const i8);

    // --- CDO Creation ---
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
            core::mem::size_of::<usize>() as u32,
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
            DbgPrint("Leyline: CDO Ready\n\0".as_ptr() as *const i8);
        }
    }

    // --- WaveRender Registration ---
    DbgPrint("Leyline: Registering WaveRender Port\n\0".as_ptr() as *const i8);
    let mut render_port: *mut u8 = core::ptr::null_mut();
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
        core::ptr::null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        return status;
    }

    let wave_render_name: [u16; 11] = [
        0x0057, 0x0061, 0x0076, 0x0065, 0x0052, 0x0065, 0x006E, 0x0064, 0x0065, 0x0072, 0x0000,
    ];
    status = PcRegisterSubdevice(device_object, wave_render_name.as_ptr(), render_port);
    if status != STATUS_SUCCESS {
        return status;
    }

    // --- WaveCapture Registration ---
    DbgPrint("Leyline: Registering WaveCapture Port\n\0".as_ptr() as *const i8);
    let mut capture_port: *mut u8 = core::ptr::null_mut();
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
        core::ptr::null_mut(),
        resource_list,
    );
    if status != STATUS_SUCCESS {
        return status;
    }

    let wave_capture_name: [u16; 12] = [
        0x0057, 0x0061, 0x0076, 0x0065, 0x0043, 0x0061, 0x0070, 0x0074, 0x0075, 0x0072, 0x0065,
        0x0000,
    ];
    status = PcRegisterSubdevice(device_object, wave_capture_name.as_ptr(), capture_port);

    if status == STATUS_SUCCESS {
        DbgPrint(
            "Leyline: StartDevice COMPLETED SUCCESSFULLY (Baseline)\n\0".as_ptr() as *const i8,
        );
    }
    status
}
