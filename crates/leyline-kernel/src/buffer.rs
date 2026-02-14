// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.
//
// This source code is provided for educational and review purposes.
// Redistribution and use in binary form without express permission is prohibited.
// See LICENSE file in the project root for full terms.

#![no_std]

/// A shared ring buffer for audio data.
pub struct RingBuffer {
    buffer: *mut u8,
    size: usize,
    write_pos: usize,
    read_pos: usize,
}

impl RingBuffer {
    /// # Safety
    /// The caller must ensure that `buffer` points to a valid memory region of at least `size` bytes.
    pub unsafe fn new(buffer: *mut u8, size: usize) -> Self {
        Self {
            buffer,
            size,
            write_pos: 0,
            read_pos: 0,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let mut written = 0;
        for &byte in data {
            // Check if buffer is full
            if (self.write_pos + 1) % self.size == self.read_pos {
                break;
            }
            // SAFETY: write_pos is always within [0, size)
            unsafe {
                *self.buffer.add(self.write_pos) = byte;
            }
            self.write_pos = (self.write_pos + 1) % self.size;
            written += 1;
        }
        written
    }

    pub fn read(&mut self, data: &mut [u8]) -> usize {
        let mut read = 0;
        for byte in data {
            // Check if buffer is empty
            if self.read_pos == self.write_pos {
                break;
            }
            // SAFETY: read_pos is always within [0, size)
            unsafe {
                *byte = *self.buffer.add(self.read_pos);
            }
            self.read_pos = (self.read_pos + 1) % self.size;
            read += 1;
        }
        read
    }

    pub fn available_write(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.size - (self.write_pos - self.read_pos) - 1
        } else {
            self.read_pos - self.write_pos - 1
        }
    }

    pub fn available_read(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            self.size - (self.read_pos - self.write_pos)
        }
    }
}
