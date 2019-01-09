use std::mem;
use std::ptr;

/// Trait for a type that can be written to a stream
pub trait Writable {
    fn write(&self, out: &mut OutStream);
}

/// Default reserved memory capacity
const DEFAULT_CAPACITY: usize = 1024;

/// Writing stream abstraction
struct OutStream {
    mem: Box<[u8]>,
    pos: usize,
}

impl OutStream {
    /// Make new instance
    pub fn new() -> Self {
        OutStream {
            mem: vec![0u8; DEFAULT_CAPACITY].into_boxed_slice(),
            pos: 0,
        }
    }

    /// Ensure that capacity is enough to fit the required number of bytes
    fn ensure_capacity(&mut self, capacity: usize) {
        if self.pos + capacity <= self.mem.len() {
            return;
        }

        let mut new_size = self.mem.len() * 2;

        while new_size < capacity {
            new_size *= 2;
        }

        let mut local: Box<[u8]> = Box::new([0u8; 0]);
        mem::swap(&mut local, &mut self.mem);

        let mut vec_mem = local.into_vec();
        vec_mem.reserve(new_size);

        self.mem = vec_mem.into_boxed_slice();
    }

    /// Get filled memory
    pub fn filled_memory(&self) -> &[u8] {
        &self.mem[0..self.pos]
    }

    /// Write i8 value to a stream
    pub fn write_i8(&mut self, value: i8) {
        self.ensure_capacity(1);

        // It is safe as safety check was performed before
        unsafe {
            self.unsafe_write_i8(value);
        }
    }

    /// Write i16 value to a stream
    pub fn write_i16(&mut self, value: i16) {
        self.ensure_capacity(2);

        // It is safe as safety check was performed before
        unsafe {
            self.unsafe_write_i16(value);
        }
    }

    /// Write i32 value to a stream
    pub fn write_i32(&mut self, value: i32) {
        self.ensure_capacity(4);

        // It is safe as safety check was performed before
        unsafe {
            self.unsafe_write_i32(value);
        }
    }

    /// Write i64 value to a stream
    pub fn write_i64(&mut self, value: i64) {
        self.ensure_capacity(8);

        // It is safe as safety check was performed before
        unsafe {
            self.unsafe_write_i64(value);
        }
    }

    /// Write string value to a stream
    pub fn write_str<'a, S: Into<&'a str>>(&mut self, value: S) {
        let value0 = value.into().as_bytes();

        self.write_u8_array(value0);
    }

    /// Write bytes to a stream
    pub fn write_u8_array<'a, A: Into<&'a [u8]>>(&mut self, value: A) {
        let value0 = value.into();

        self.ensure_capacity(4 + value0.len());

        // It is safe as safety check was performed before
        unsafe {
            self.unsafe_write_i32(value0.len() as i32);
            self.unsafe_write_bytes(value0);
        }
    }

    /// Get mutable pointer to a free space
    /// Unchecked
    #[inline(always)]
    unsafe fn mut_ptr_to_free_space(&mut self) -> *mut u8 {
        self.mem.as_mut_ptr().add(self.pos)
    }

    /// Write i8 value without capacity checks
    unsafe fn unsafe_write_i8(&mut self, value: i8) {
        let dst = self.mut_ptr_to_free_space();

        *dst = (value & 0xFF) as u8;

        self.pos += 1;
    }

    /// Write i16 value without capacity checks
    unsafe fn unsafe_write_i16(&mut self, value: i16) {
        let dst = self.mut_ptr_to_free_space();

        *dst = (value & 0xFF) as u8;
        *dst.add(1) = (value >> 8 & 0xFF) as u8;

        self.pos += 2;
    }

    /// Write i32 value without capacity checks
    unsafe fn unsafe_write_i32(&mut self, value: i32) {
        let dst = self.mut_ptr_to_free_space();

        *dst = (value & 0xFF) as u8;
        *dst.add(1) = (value >> 8 & 0xFF) as u8;
        *dst.add(2) = (value >> 16 & 0xFF) as u8;
        *dst.add(3) = (value >> 24 & 0xFF) as u8;

        self.pos += 4;
    }

    /// Write i64 value without capacity checks
    unsafe fn unsafe_write_i64(&mut self, value: i64) {
        let dst = self.mut_ptr_to_free_space();

        *dst = (value & 0xFF) as u8;
        *dst.add(1) = (value >> 8 & 0xFF) as u8;
        *dst.add(2) = (value >> 16 & 0xFF) as u8;
        *dst.add(3) = (value >> 24 & 0xFF) as u8;
        *dst.add(4) = (value >> 32 & 0xFF) as u8;
        *dst.add(5) = (value >> 40 & 0xFF) as u8;
        *dst.add(6) = (value >> 48 & 0xFF) as u8;
        *dst.add(7) = (value >> 56 & 0xFF) as u8;

        self.pos += 8;
    }

    /// Write bytes without capacity checks
    unsafe fn unsafe_write_bytes(&mut self, bytes: &[u8]) {
        let dst = self.mut_ptr_to_free_space();
        let src = bytes.as_ptr();

        ptr::copy(src, dst, bytes.len());

        self.pos += bytes.len();
    }
}

#[test]
fn test_write_i8() {
    let mut out = OutStream::new();

    out.write_i8(0xF4u8 as i8);

    let mem = out.filled_memory();

    assert_eq!(mem.len(), 1);
    assert_eq!(mem[0], 0xF4);
}

#[test]
fn test_write_i16() {
    let mut out = OutStream::new();

    out.write_i16(0x4321);

    let mem = out.filled_memory();

    assert_eq!(mem.len(), 2);
    assert_eq!(mem[0], 0x21);
    assert_eq!(mem[1], 0x43);
}

#[test]
fn test_write_i32() {
    let mut out = OutStream::new();

    out.write_i32(0x11223344);

    let mem = out.filled_memory();

    assert_eq!(mem.len(), 4);
    assert_eq!(mem[0], 0x44);
    assert_eq!(mem[1], 0x33);
    assert_eq!(mem[2], 0x22);
    assert_eq!(mem[3], 0x11);
}

#[test]
fn test_write_i64() {
    let mut out = OutStream::new();

    out.write_i64(0xEFCDAB8967452301u64 as i64);

    let mem = out.filled_memory();

    assert_eq!(mem.len(), 8);
    assert_eq!(mem[0], 0x01);
    assert_eq!(mem[1], 0x23);
    assert_eq!(mem[2], 0x45);
    assert_eq!(mem[3], 0x67);
    assert_eq!(mem[4], 0x89);
    assert_eq!(mem[5], 0xAB);
    assert_eq!(mem[6], 0xCD);
    assert_eq!(mem[7], 0xEF);
}

#[test]
fn test_write_str() {
    let mut out = OutStream::new();

    let value = "Hello World!";

    out.write_str(value);

    let mem = out.filled_memory();

    assert_eq!(mem.len(), 4 + value.len());

    assert_eq!(mem[0], value.len() as u8);
    assert_eq!(mem[1], 0);
    assert_eq!(mem[2], 0);
    assert_eq!(mem[3], 0);

    assert_eq!(mem[4], 'H' as u8);
    assert_eq!(mem[5], 'e' as u8);
    assert_eq!(mem[6], 'l' as u8);
    assert_eq!(mem[7], 'l' as u8);
    assert_eq!(mem[8], 'o' as u8);
    assert_eq!(mem[9], ' ' as u8);
    assert_eq!(mem[10], 'W' as u8);
    assert_eq!(mem[11], 'o' as u8);
    assert_eq!(mem[12], 'r' as u8);
    assert_eq!(mem[13], 'l' as u8);
    assert_eq!(mem[14], 'd' as u8);
    assert_eq!(mem[15], '!' as u8);
}
