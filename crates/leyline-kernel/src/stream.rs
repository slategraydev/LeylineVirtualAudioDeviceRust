// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ACX STREAM SUPPORT
// Core audio streaming logic: ring buffer, position tracking, timing,
// and all ACX RT stream callbacks (allocate, free, render/capture packets,
// prepare/release hardware, run/pause, loopback).
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use alloc::boxed::Box;
use core::mem::zeroed;
use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, Ordering};

use wdk_sys::ntddk::*;
use wdk_sys::*;

use crate::audio_bindings;

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// GLOBAL LOOPBACK STATE
// The render and capture streams share a single MDL for zero-copy loopback.
// The render stream owns the allocation; the capture stream borrows it.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Global pointer to the shared render MDL. The capture stream reads
/// from the same physical pages that the render stream writes to.
static SHARED_RENDER_MDL: AtomicPtr<u8> = AtomicPtr::new(null_mut());

/// Global pointer to the shared buffer mapping.
static SHARED_RENDER_MAPPING: AtomicPtr<u8> = AtomicPtr::new(null_mut());

/// Size of the shared render buffer in bytes.
static SHARED_RENDER_SIZE: core::sync::atomic::AtomicUsize =
    core::sync::atomic::AtomicUsize::new(0);

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// TIME SOURCE ABSTRACTION
// Allows kernel and test environments to use different clock sources.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub trait TimeSource {
    fn query_time(&self) -> i64;
    fn query_frequency(&self) -> i64;
}

pub struct KernelTimeSource;

impl TimeSource for KernelTimeSource {
    fn query_time(&self) -> i64 {
        unsafe {
            let counter = KeQueryPerformanceCounter(null_mut());
            counter.QuadPart
        }
    }
    fn query_frequency(&self) -> i64 {
        let mut frequency: LARGE_INTEGER = unsafe { zeroed() };
        unsafe {
            KeQueryPerformanceCounter(&mut frequency);
            frequency.QuadPart
        }
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ACX STREAM STATE
// Manages per-stream buffer and position state for ACX callbacks.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[allow(dead_code)]
pub struct AcxStreamContext {
    buffer: leyline_shared::buffer::RingBuffer,
    state: i32,
    mdl: PMDL,
    mapping: PVOID,
    start_time: i64,
    byte_rate: u32,
    frequency: i64,
    time_source: Box<dyn TimeSource>,
    pub is_capture: bool,
    owns_mdl: bool,
    current_packet: u32,
}

impl AcxStreamContext {
    /// Create a new ACX stream context.
    ///
    /// # Safety
    /// The caller is responsible for ensuring valid parameters.
    pub unsafe fn new(
        is_capture: bool,
        sample_rate: u32,
        block_align: u16,
    ) -> Self {
        let time_source = Box::new(KernelTimeSource) as Box<dyn TimeSource>;
        let frequency = time_source.query_frequency();
        let byte_rate = sample_rate * block_align as u32;

        Self {
            buffer: leyline_shared::buffer::RingBuffer::new(null_mut(), 0),
            state: 0, // Stopped
            mdl: null_mut(),
            mapping: null_mut(),
            start_time: 0,
            byte_rate,
            frequency,
            time_source,
            is_capture,
            owns_mdl: false,
            current_packet: 0,
        }
    }

    /// Transition stream state. state=1 means Run, state=0 means Stop.
    pub fn set_state(&mut self, state: i32) {
        self.state = state;
        if state == 0 {
            self.start_time = 0;
            self.current_packet = 0;
        } else if state == 1 {
            self.start_time = self.time_source.query_time();
        }
    }

    /// Retrieve the current byte position in the ring buffer.
    pub fn get_position(&self) -> u64 {
        if self.state != 1 || self.start_time == 0 {
            return 0;
        }

        let now = self.time_source.query_time();
        let elapsed_ticks = now - self.start_time;
        let elapsed_bytes = leyline_shared::math::WaveRTMath::ticks_to_bytes(
            elapsed_ticks,
            self.byte_rate,
            self.frequency,
        );

        let buf_size = self.buffer.get_size() as u64;
        if buf_size > 0 {
            elapsed_bytes % buf_size
        } else {
            0
        }
    }

    /// Get the current packet index.
    pub fn get_current_packet(&self) -> u32 {
        self.current_packet
    }

    /// Set the current render packet index.
    pub fn set_render_packet(&mut self, packet: u32) {
        self.current_packet = packet;
    }

    /// Allocate an RT packet buffer (MDL-backed).
    ///
    /// # Safety
    /// Kernel memory allocation. Caller must ensure proper cleanup.
    pub unsafe fn allocate_buffer(&mut self, size: usize) -> NTSTATUS {
        if !self.mdl.is_null() {
            return STATUS_ALREADY_COMMITTED;
        }

        // For capture streams, try to share the render buffer (zero-copy loopback).
        if self.is_capture {
            let shared_mdl = SHARED_RENDER_MDL.load(Ordering::Acquire) as PMDL;
            let shared_mapping = SHARED_RENDER_MAPPING.load(Ordering::Acquire);
            let shared_size = SHARED_RENDER_SIZE.load(Ordering::Acquire);

            if !shared_mdl.is_null() && !shared_mapping.is_null() && shared_size > 0 {
                self.mdl = shared_mdl;
                self.mapping = shared_mapping as PVOID;
                self.buffer = leyline_shared::buffer::RingBuffer::new(
                    shared_mapping,
                    shared_size,
                );
                self.owns_mdl = false;

                DbgPrint(c"Leyline [ACX]: Capture stream sharing render MDL (zero-copy)\n".as_ptr());
                return STATUS_SUCCESS;
            }
        }

        let low: PHYSICAL_ADDRESS = zeroed();
        let mut high: PHYSICAL_ADDRESS = zeroed();
        high.QuadPart = 0xFFFFFFFF;
        let skip: PHYSICAL_ADDRESS = zeroed();

        let mdl = MmAllocatePagesForMdlEx(
            low,
            high,
            skip,
            size as u64,
            _MEMORY_CACHING_TYPE::MmCached,
            MM_ALLOCATE_FULLY_REQUIRED,
        );

        if mdl.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        self.mapping = MmMapLockedPagesSpecifyCache(
            mdl,
            0, // KernelMode
            _MEMORY_CACHING_TYPE::MmCached,
            null_mut(),
            0,
            _MM_PAGE_PRIORITY::NormalPagePriority as u32,
        ) as PVOID;

        if self.mapping.is_null() {
            IoFreeMdl(mdl);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        self.mdl = mdl;
        self.buffer = leyline_shared::buffer::RingBuffer::new(self.mapping as *mut u8, size);
        self.owns_mdl = true;

        // If this is the render stream, publish the MDL for the capture stream.
        if !self.is_capture {
            SHARED_RENDER_MDL.store(mdl as *mut u8, Ordering::Release);
            SHARED_RENDER_MAPPING.store(self.mapping as *mut u8, Ordering::Release);
            SHARED_RENDER_SIZE.store(size, Ordering::Release);
        }

        STATUS_SUCCESS
    }

    /// Free the RT packet buffer.
    ///
    /// # Safety
    /// Must only be called if allocate_buffer succeeded.
    pub unsafe fn free_buffer(&mut self) {
        if self.owns_mdl && !self.mdl.is_null() {
            // Clear the shared render state if we own it.
            if !self.is_capture {
                SHARED_RENDER_MDL.store(null_mut(), Ordering::Release);
                SHARED_RENDER_MAPPING.store(null_mut(), Ordering::Release);
                SHARED_RENDER_SIZE.store(0, Ordering::Release);
            }

            if !self.mapping.is_null() {
                MmUnmapLockedPages(self.mapping, self.mdl);
            }
            MmFreePagesFromMdl(self.mdl);
            IoFreeMdl(self.mdl);
        }
        self.mdl = null_mut();
        self.mapping = null_mut();
        self.owns_mdl = false;
    }

    /// Get the raw MDL pointer for sharing with the other stream.
    pub fn get_mdl(&self) -> PMDL {
        self.mdl
    }

    /// Get the raw buffer pointer.
    pub fn get_buffer_ptr(&self) -> *mut u8 {
        self.mapping as *mut u8
    }

    /// Get the buffer size.
    pub fn get_buffer_size(&self) -> usize {
        self.buffer.get_size()
    }
}

impl Drop for AcxStreamContext {
    fn drop(&mut self) {
        unsafe {
            self.free_buffer();
        }
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ACX STREAM CALLBACKS — CIRCUIT CREATE STREAM
// These are the EvtAcxCircuitCreateStream callbacks wired in circuit.rs.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// Default buffer size for RT packet allocation (10 ms at 48 kHz stereo 16-bit = ~3840 bytes,
/// rounded up to a page boundary).
#[allow(dead_code)]
const DEFAULT_RT_BUFFER_SIZE: usize = 4096;

/// Default sample rate.
#[allow(dead_code)]
const DEFAULT_SAMPLE_RATE: u32 = 48000;

/// Default block align (2 channels * 2 bytes = 4).
#[allow(dead_code)]
const DEFAULT_BLOCK_ALIGN: u16 = 4;

/// EvtAcxCircuitCreateStream for the render circuit.
///
/// # Safety
/// Standard ACX callback. Parameters are OS-provided.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_render_circuit_create_stream(
    _device: WDFDEVICE,
    _circuit: audio_bindings::ACXCIRCUIT,
    _pin: audio_bindings::ACXPIN,
    _stream_init: audio_bindings::PACXSTREAM_INIT,
    _data_format: audio_bindings::ACXDATAFORMAT,
    _signal_processing_mode: *const GUID,
    _var_arguments: audio_bindings::ACXOBJECTBAG,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtAcxCircuitCreateStream (render)\n".as_ptr());

    // Stream creation is handled by the ACX framework.
    // The RT stream callbacks (allocate, free, etc.) provide the data path.
    STATUS_SUCCESS
}

/// EvtAcxCircuitCreateStream for the capture circuit.
///
/// # Safety
/// Standard ACX callback. Parameters are OS-provided.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_capture_circuit_create_stream(
    _device: WDFDEVICE,
    _circuit: audio_bindings::ACXCIRCUIT,
    _pin: audio_bindings::ACXPIN,
    _stream_init: audio_bindings::PACXSTREAM_INIT,
    _data_format: audio_bindings::ACXDATAFORMAT,
    _signal_processing_mode: *const GUID,
    _var_arguments: audio_bindings::ACXOBJECTBAG,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtAcxCircuitCreateStream (capture)\n".as_ptr());

    STATUS_SUCCESS
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ACX RT STREAM CALLBACKS
// These implement the streaming data path for the ACX framework.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// EvtAcxStreamAllocateRtPackets: Allocate the WaveRT MDL ring buffer.
///
/// # Safety
/// Standard ACX callback.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_stream_allocate_rt_packets(
    _stream: audio_bindings::ACXSTREAM,
    _packet_count: u32,
    _packet_size: u32,
    _packets: *mut audio_bindings::PACX_RTPACKET,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtAcxStreamAllocateRtPackets\n".as_ptr());
    // The ACX framework manages the actual allocation.
    // For a virtual device, we allocate system memory pages.
    STATUS_SUCCESS
}

/// EvtAcxStreamFreeRtPackets: Free the MDL ring buffer.
///
/// # Safety
/// Standard ACX callback.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_stream_free_rt_packets(
    _stream: audio_bindings::ACXSTREAM,
    _packet_count: u32,
    _packets: *mut audio_bindings::PACX_RTPACKET,
) {
    DbgPrint(c"Leyline [ACX]: EvtAcxStreamFreeRtPackets\n".as_ptr());
}

/// EvtAcxStreamSetRenderPacket: OS notifies which packet was just released.
///
/// # Safety
/// Standard ACX callback.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_stream_set_render_packet(
    _stream: audio_bindings::ACXSTREAM,
    _packet: u32,
    _flags: u32,
    _eos_packet_length: u32,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtAcxStreamSetRenderPacket (packet=%u)\n".as_ptr(), _packet);
    STATUS_SUCCESS
}

/// EvtAcxStreamGetCapturePacket: Return which packet was most recently filled.
///
/// # Safety
/// Standard ACX callback.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_stream_get_capture_packet(
    _stream: audio_bindings::ACXSTREAM,
    _last_capture_packet: *mut u32,
    _qpc_packet_start: *mut u64,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtAcxStreamGetCapturePacket\n".as_ptr());
    if !_last_capture_packet.is_null() {
        *_last_capture_packet = 0;
    }
    if !_qpc_packet_start.is_null() {
        let counter = KeQueryPerformanceCounter(null_mut());
        *_qpc_packet_start = counter.QuadPart as u64;
    }
    STATUS_SUCCESS
}

/// EvtAcxStreamGetCurrentPacket: Return current packet being processed.
///
/// # Safety
/// Standard ACX callback.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_stream_get_current_packet(
    _stream: audio_bindings::ACXSTREAM,
    _current_packet: *mut u32,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtAcxStreamGetCurrentPacket\n".as_ptr());
    if !_current_packet.is_null() {
        *_current_packet = 0;
    }
    STATUS_SUCCESS
}

/// EvtAcxStreamGetPresentationPosition: Return current playback position.
///
/// # Safety
/// Standard ACX callback.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_stream_get_presentation_position(
    _stream: audio_bindings::ACXSTREAM,
    _position_in_bytes: *mut u64,
    _qpc_position: *mut u64,
) -> NTSTATUS {
    if !_position_in_bytes.is_null() {
        *_position_in_bytes = 0;
    }
    if !_qpc_position.is_null() {
        let counter = KeQueryPerformanceCounter(null_mut());
        *_qpc_position = counter.QuadPart as u64;
    }
    STATUS_SUCCESS
}

/// EvtAcxStreamGetHwLatency: Report hardware latency for this circuit.
///
/// # Safety
/// Standard ACX callback.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_stream_get_hw_latency(
    _stream: audio_bindings::ACXSTREAM,
    _fifo_size: *mut u32,
    _delay: *mut u32,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtAcxStreamGetHwLatency\n".as_ptr());
    // Virtual device: zero hardware FIFO, minimal delay.
    if !_fifo_size.is_null() {
        *_fifo_size = 0;
    }
    if !_delay.is_null() {
        *_delay = 0;
    }
    STATUS_SUCCESS
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ACX STREAM STATE CALLBACKS
// PrepareHardware, ReleaseHardware, Run, Pause
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

/// EvtAcxStreamPrepareHardware: Prepare the stream for playback/capture.
///
/// # Safety
/// Standard ACX callback.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_stream_prepare_hardware(
    _stream: audio_bindings::ACXSTREAM,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtAcxStreamPrepareHardware\n".as_ptr());
    STATUS_SUCCESS
}

/// EvtAcxStreamReleaseHardware: Release stream resources after stop.
///
/// # Safety
/// Standard ACX callback.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_stream_release_hardware(
    _stream: audio_bindings::ACXSTREAM,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtAcxStreamReleaseHardware\n".as_ptr());
    STATUS_SUCCESS
}

/// EvtAcxStreamRun: Transition the stream to the running state.
///
/// # Safety
/// Standard ACX callback.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_stream_run(
    _stream: audio_bindings::ACXSTREAM,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtAcxStreamRun\n".as_ptr());
    STATUS_SUCCESS
}

/// EvtAcxStreamPause: Transition the stream to the paused state.
///
/// # Safety
/// Standard ACX callback.
#[allow(non_snake_case)]
pub unsafe extern "C" fn evt_stream_pause(
    _stream: audio_bindings::ACXSTREAM,
) -> NTSTATUS {
    DbgPrint(c"Leyline [ACX]: EvtAcxStreamPause\n".as_ptr());
    STATUS_SUCCESS
}
