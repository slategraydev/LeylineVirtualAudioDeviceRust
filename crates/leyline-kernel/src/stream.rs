// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// ACX STREAM SUPPORT
// Core audio streaming logic: ring buffer, position tracking, and timing.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

use alloc::boxed::Box;
use core::mem::zeroed;
use core::ptr::null_mut;

use wdk_sys::ntddk::*;
use wdk_sys::*;

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
        }
    }

    /// Transition stream state. state=1 means Run, state=0 means Stop.
    pub fn set_state(&mut self, state: i32) {
        self.state = state;
        if state == 0 {
            self.start_time = 0;
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

    /// Allocate an RT packet buffer (MDL-backed).
    ///
    /// # Safety
    /// Kernel memory allocation. Caller must ensure proper cleanup.
    pub unsafe fn allocate_buffer(&mut self, size: usize) -> NTSTATUS {
        if !self.mdl.is_null() {
            return STATUS_ALREADY_COMMITTED;
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

        STATUS_SUCCESS
    }

    /// Free the RT packet buffer.
    ///
    /// # Safety
    /// Must only be called if allocate_buffer succeeded.
    pub unsafe fn free_buffer(&mut self) {
        if self.owns_mdl && !self.mdl.is_null() {
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
