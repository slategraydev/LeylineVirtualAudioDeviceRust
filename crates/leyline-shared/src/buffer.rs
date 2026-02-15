// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

/// A single byte is reserved to distinguish between full and empty states.
const RESERVED_BYTE: usize = 1;

/// A shared ring buffer for audio data.
pub struct RingBuffer {
    buffer: *mut u8,
    size: usize,
    write_pos: usize,
    read_pos: usize,
}

impl RingBuffer {
    pub fn get_ptr(&self) -> *mut u8 {
        self.buffer
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub unsafe fn new(buffer: *mut u8, size: usize) -> Self {
        Self {
            buffer,
            size,
            write_pos: 0,
            read_pos: 0,
        }
    }

    pub unsafe fn rebase(&mut self, buffer: *mut u8, size: usize) {
        self.buffer = buffer;
        self.size = size;
        self.write_pos = 0;
        self.read_pos = 0;
    }

    pub fn write(&mut self, data: &[u8]) -> usize {
        let available = self.available_write();
        let to_write = core::cmp::min(data.len(), available);

        if to_write == 0 {
            return 0;
        }

        let first_part = core::cmp::min(to_write, self.size - self.write_pos);

        unsafe {
            core::ptr::copy_nonoverlapping(
                data.as_ptr(),
                self.buffer.add(self.write_pos),
                first_part,
            );
        }

        if first_part < to_write {
            let second_part = to_write - first_part;
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

    pub fn read(&mut self, data: &mut [u8]) -> usize {
        let available = self.available_read();
        let to_read = core::cmp::min(data.len(), available);

        if to_read == 0 {
            return 0;
        }

        let first_part = core::cmp::min(to_read, self.size - self.read_pos);

        unsafe {
            core::ptr::copy_nonoverlapping(
                self.buffer.add(self.read_pos),
                data.as_mut_ptr(),
                first_part,
            );
        }

        if first_part < to_read {
            let second_part = to_read - first_part;
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

    pub fn reset(&mut self) {
        self.write_pos = 0;
        self.read_pos = 0;
    }

    pub fn available_write(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.size - (self.write_pos - self.read_pos) - RESERVED_BYTE
        } else {
            self.read_pos - self.write_pos - RESERVED_BYTE
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

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::vec;

    #[test]
    fn test_ring_buffer_basic() {
        let mut storage = vec![0u8; 1024];
        let mut rb = unsafe { RingBuffer::new(storage.as_mut_ptr(), 1024) };

        assert_eq!(rb.available_write(), 1023);
        assert_eq!(rb.available_read(), 0);

        let data = [1, 2, 3, 4, 5];
        let written = rb.write(&data);
        assert_eq!(written, 5);
        assert_eq!(rb.available_read(), 5);

        let mut read_buf = [0u8; 5];
        let read = rb.read(&mut read_buf);
        assert_eq!(read, 5);
        assert_eq!(read_buf, data);
    }

    #[test]
    fn test_ring_buffer_wrap() {
        let mut storage = vec![0u8; 10];
        let mut rb = unsafe { RingBuffer::new(storage.as_mut_ptr(), 10) };

        rb.write(&[1, 2, 3, 4, 5, 6, 7]);
        let mut tmp = [0u8; 5];
        rb.read(&mut tmp);

        let written = rb.write(&[8, 9, 10, 11, 12, 13]);
        assert_eq!(written, 6);
        assert_eq!(rb.available_read(), 8);

        let mut final_buf = [0u8; 8];
        rb.read(&mut final_buf);
        assert_eq!(final_buf, [6, 7, 8, 9, 10, 11, 12, 13]);
    }
}
