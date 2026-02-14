// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

// ============================================================================
// Constants
// ============================================================================
// Standard constants used for buffer arithmetic and state management.

/// A single byte is reserved to distinguish between full and empty states
/// in this ring buffer implementation.
const RESERVED_BYTE: usize = 1;

// ============================================================================
// Ring Buffer Structure
// ============================================================================
// A shared ring buffer for audio data, providing lock-free read/write
// operations suitable for real-time kernel-mode audio streaming.

/// A shared ring buffer for audio data.
pub struct RingBuffer {
    buffer: *mut u8,
    size: usize,
    write_pos: usize,
    read_pos: usize,
}

// ============================================================================
// Ring Buffer Implementation
// ============================================================================
// Core logic for buffer lifecycle, I/O operations, and position tracking.

impl RingBuffer {
    /// Returns the raw pointer to the underlying buffer.
    pub fn get_ptr(&self) -> *mut u8 {
        self.buffer
    }

    /// # Safety
    /// The caller must ensure that `buffer` points to a valid memory region
    /// of at least `size` bytes. The lifetime of the buffer must exceed
    /// the lifetime of the `RingBuffer` object.
    pub unsafe fn new(buffer: *mut u8, size: usize) -> Self {
        Self {
            buffer,
            size,
            write_pos: 0,
            read_pos: 0,
        }
    }

    /// Updates the buffer location and size.
    /// # Safety
    /// The caller must ensure the new buffer is valid.
    pub unsafe fn rebase(&mut self, buffer: *mut u8, size: usize) {
        self.buffer = buffer;
        self.size = size;
        self.write_pos = 0;
        self.read_pos = 0;
    }

    /// Writes data to the ring buffer.
    /// Returns the number of bytes actually written.
    pub fn write(&mut self, data: &[u8]) -> usize {
        let available: usize;
        let to_write: usize;
        let first_part: usize;
        let second_part: usize;

        available = self.available_write();
        to_write = core::cmp::min(data.len(), available);

        if to_write == 0 {
            return 0;
        }

        first_part = core::cmp::min(to_write, self.size - self.write_pos);

        // SAFETY: The offsets are calculated to stay within the allocated
        // bounds of the buffer.
        unsafe {
            core::ptr::copy_nonoverlapping(
                data.as_ptr(),
                self.buffer.add(self.write_pos),
                first_part,
            );
        }

        if first_part < to_write {
            second_part = to_write - first_part;

            // SAFETY: Wraparound write to the beginning of the buffer.
            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(first_part),
                    self.buffer,
                    second_part,
                );
            }
        }

        self.write_pos = (self.write_pos + to_write) % self.size;
        to_write
    }

    /// Reads data from the ring buffer.
    /// Returns the number of bytes actually read.
    pub fn read(&mut self, data: &mut [u8]) -> usize {
        let available: usize;
        let to_read: usize;
        let first_part: usize;
        let second_part: usize;

        available = self.available_read();
        to_read = core::cmp::min(data.len(), available);

        if to_read == 0 {
            return 0;
        }

        first_part = core::cmp::min(to_read, self.size - self.read_pos);

        // SAFETY: Sequential read from the current position.
        unsafe {
            core::ptr::copy_nonoverlapping(
                self.buffer.add(self.read_pos),
                data.as_mut_ptr(),
                first_part,
            );
        }

        if first_part < to_read {
            second_part = to_read - first_part;

            // SAFETY: Wraparound read from the beginning of the buffer.
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.buffer,
                    data.as_mut_ptr().add(first_part),
                    second_part,
                );
            }
        }

        self.read_pos = (self.read_pos + to_read) % self.size;
        to_read
    }

    /// Resets the buffer positions to zero.
    pub fn reset(&mut self) {
        self.write_pos = 0;
        self.read_pos = 0;
    }

    /// Calculates the number of bytes available for writing.
    pub fn available_write(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.size - (self.write_pos - self.read_pos) - RESERVED_BYTE
        } else {
            self.read_pos - self.write_pos - RESERVED_BYTE
        }
    }

    /// Calculates the number of bytes available for reading.
    pub fn available_read(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            self.size - (self.read_pos - self.write_pos)
        }
    }
}
