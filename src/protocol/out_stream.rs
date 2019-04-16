use std::cell::Cell;
use std::mem;
use std::ptr;
use std::thread;

use super::growing_buffer::GrowingBuffer;
use crate::protocol::header;

/// Trait for a type that can be written to a stream
pub trait Writable {
    fn write(&self, out: &OutStream);
}

/// Trait for a type that can be packed into a byte buffer
pub trait Pack {
    fn pack(&self) -> Box<[u8]>;
}

impl<'a, T: Writable + 'a> Pack for T {
    /// Pack any Writable value into boxed slice
    fn pack(&self) -> Box<[u8]> {
        let stream = OutStream::new();

        let len = stream.reserve_len();

        self.write(&stream);

        len.set();

        stream.into_memory()
    }
}

/// Writing stream abstraction
pub struct OutStream {
    buffer: GrowingBuffer,
    pos: Cell<usize>,
}

impl OutStream {
    /// Make new instance
    pub fn new() -> Self {
        Self {
            buffer: GrowingBuffer::new(),
            pos: Cell::new(0),
        }
    }

    /// Make new instance with required capacity
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            buffer: GrowingBuffer::with_capacity(cap),
            pos: Cell::new(0),
        }
    }

    /// Ensure that capacity is enough to fit the required number of bytes
    fn ensure_capacity(&self, capacity: usize) {
        self.buffer.reserve(self.pos.get() + capacity);
    }

    /// Get filled memory
    pub fn into_memory(self) -> Box<[u8]> {
        self.buffer.into_memory(self.pos.get())
    }

    /// Get current position in stream
    pub fn position(&self) -> usize {
        self.pos.get()
    }

    /// Write i8 value to a stream
    pub fn write_i8(&self, value: i8) {
        self.ensure_capacity(1);

        // It is safe as safety check was performed before
        unsafe {
            self.unsafe_write_i8(value);
        }
    }

    /// Write i16 value to a stream
    pub fn write_i16(&self, value: i16) {
        self.ensure_capacity(2);

        // It is safe as safety check was performed before
        unsafe {
            self.unsafe_write_i16(value);
        }
    }

    /// Write i32 value to a stream
    pub fn write_i32(&self, value: i32) {
        self.ensure_capacity(4);

        // It is safe as safety check was performed before
        unsafe {
            self.unsafe_write_i32(value);
        }
    }

    /// Write i64 value to a stream
    pub fn write_i64(&self, value: i64) {
        self.ensure_capacity(8);

        // It is safe as safety check was performed before
        unsafe {
            self.unsafe_write_i64(value);
        }
    }

    /// Write string value to a stream
    pub fn write_str<S: AsRef<str>>(&self, value: S) {
        let value0 = value.as_ref().as_bytes();

        self.ensure_capacity(1 + 4 + value0.len());

        // It is safe as safety check was performed before
        unsafe {
            self.unsafe_write_i8(header::STRING);
            self.unsafe_write_u8_array(value0);
        }
    }

    /// Write string value to a stream
    pub fn write_str_raw<S: AsRef<str>>(&self, value: S) {
        let value0 = value.as_ref().as_bytes();

        self.write_u8_array_raw(value0);
    }

    /// Write bytes to a stream
    pub fn write_u8_array_raw<A: AsRef<[u8]>>(&self, value: A) {
        let value0 = value.as_ref();

        self.ensure_capacity(4 + value0.len());

        // It is safe as safety check was performed before
        unsafe {
            self.unsafe_write_u8_array(value0);
        }
    }

    /// Reserve a space in a stream for a i32 value.
    pub fn reserve_i32(&self) -> ReservedI32 {
        self.ensure_capacity(4);

        let reserved = ReservedI32::new(self);

        self.add_pos(4);

        reserved
    }

    /// Reserve a space in a stream for a i32 value which will be lately set to
    /// a length of the block of data.
    pub fn reserve_len(&self) -> ReservedLen {
        self.ensure_capacity(4);

        let reserved = ReservedLen::new(self);

        self.add_pos(4);

        reserved
    }

    /// Write i8 value without capacity checks
    unsafe fn unsafe_write_i8(&self, value: i8) {
        let dst = self.mut_ptr_to_free_space();

        *dst = value as u8 & 0xFFu8;

        self.add_pos(1);
    }

    /// Write i16 value without capacity checks
    unsafe fn unsafe_write_i16(&self, value: i16) {
        let dst = self.mut_ptr_to_free_space();

        *dst = (value & 0xFF) as u8;
        *dst.add(1) = (value >> 8 & 0xFF) as u8;

        self.add_pos(2);
    }

    /// Write i32 value without capacity checks
    unsafe fn unsafe_write_i32(&self, value: i32) {
        self.unsafe_write_i32_to_pos(self.pos.get(), value);

        self.add_pos(4);
    }

    /// Write i32 value to a specific position without capacity checks
    unsafe fn unsafe_write_i32_to_pos(&self, pos: usize, value: i32) {
        let dst = self.mut_ptr_to_position(pos);

        *dst = (value & 0xFF) as u8;
        *dst.add(1) = (value >> 8 & 0xFF) as u8;
        *dst.add(2) = (value >> 16 & 0xFF) as u8;
        *dst.add(3) = (value >> 24 & 0xFF) as u8;
    }

    /// Write i64 value without capacity checks
    unsafe fn unsafe_write_i64(&self, value: i64) {
        let dst = self.mut_ptr_to_free_space();

        *dst = (value & 0xFF) as u8;
        *dst.add(1) = (value >> 8 & 0xFF) as u8;
        *dst.add(2) = (value >> 16 & 0xFF) as u8;
        *dst.add(3) = (value >> 24 & 0xFF) as u8;
        *dst.add(4) = (value >> 32 & 0xFF) as u8;
        *dst.add(5) = (value >> 40 & 0xFF) as u8;
        *dst.add(6) = (value >> 48 & 0xFF) as u8;
        *dst.add(7) = (value >> 56 & 0xFF) as u8;

        self.add_pos(8);
    }

    /// Write bytes to a stream without checks
    unsafe fn unsafe_write_u8_array(&self, array: &[u8]) {
        self.unsafe_write_i32(array.len() as i32);
        self.unsafe_write_bytes(array);
    }

    /// Write bytes without capacity checks
    unsafe fn unsafe_write_bytes(&self, bytes: &[u8]) {
        let dst = self.mut_ptr_to_free_space();
        let src = bytes.as_ptr();

        ptr::copy(src, dst, bytes.len());

        self.add_pos(bytes.len());
    }

    /// Get mutable pointer to a free space
    /// Unchecked
    #[inline(always)]
    unsafe fn mut_ptr_to_free_space(&self) -> *mut u8 {
        self.mut_ptr_to_position(self.pos.get())
    }

    /// Get mutable pointer to a free space
    /// Unchecked
    #[inline(always)]
    unsafe fn mut_ptr_to_position(&self, pos: usize) -> *mut u8 {
        self.mut_ptr().add(pos)
    }

    /// Get mutable pointer
    /// Unchecked
    #[inline(always)]
    unsafe fn mut_ptr(&self) -> *mut u8 {
        self.buffer.mut_ptr()
    }

    /// Increase position
    #[inline(always)]
    fn add_pos(&self, add: usize) {
        self.pos.set(self.pos.get() + add);
    }
}

struct ShouldNotDrop;

impl Drop for ShouldNotDrop {
    fn drop(&mut self) {
        // Panic results in unwind and subsequent call to drop(), so we need to
        // ensure here we are not currently panicking, to avoid aborting of the
        // whole process.
        assert!(thread::panicking(),
            "Fatal error: Reserved value was not set properly. Panicking to prevent undefined behaviour");
    }
}

pub struct ReservedI32<'a> {
    stream: &'a OutStream,
    pos: usize,
    snd: ShouldNotDrop,
}

impl<'a> ReservedI32<'a> {
    /// Make new instance
    fn new<'b: 'a>(stream: &'b OutStream) -> Self {
        Self {
            stream,
            pos: stream.pos.get(),
            snd: ShouldNotDrop,
        }
    }

    /// Set value. Consumes an instance.
    pub fn set(self, value: i32) {
        unsafe {
            self.stream.unsafe_write_i32_to_pos(self.pos, value);

            mem::forget(self.snd);
        }
    }
}

pub struct ReservedLen<'a> {
    val: ReservedI32<'a>,
}

impl<'a> ReservedLen<'a> {
    /// Make new instance
    fn new<'b: 'a>(stream: &'b OutStream) -> Self {
        Self {
            val: ReservedI32::new(stream),
        }
    }

    /// Set value. Consumes an instance.
    pub fn set(self) {
        let len = self.val.stream.pos.get() - self.val.pos - 4;
        self.val.set(len as i32);
    }
}

#[test]
fn test_write_i8() {
    let out = OutStream::new();

    out.write_i8(0xF4u8 as i8);

    let mem = out.into_memory();

    assert_eq!(mem.len(), 1);
    assert_eq!(mem[0], 0xF4);
}

#[test]
fn test_write_i16() {
    let out = OutStream::new();

    out.write_i16(0x4321);

    let mem = out.into_memory();

    assert_eq!(mem.len(), 2);
    assert_eq!(mem[0], 0x21);
    assert_eq!(mem[1], 0x43);
}

#[test]
fn test_write_i32() {
    let out = OutStream::new();

    out.write_i32(0x11223344);

    let mem = out.into_memory();

    assert_eq!(mem.len(), 4);
    assert_eq!(mem[0], 0x44);
    assert_eq!(mem[1], 0x33);
    assert_eq!(mem[2], 0x22);
    assert_eq!(mem[3], 0x11);
}

#[test]
fn test_write_i64() {
    let out = OutStream::new();

    out.write_i64(0xEFCDAB8967452301u64 as i64);

    let mem = out.into_memory();

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
fn test_write_str_raw() {
    let out = OutStream::new();

    let value = "Hello World!";

    out.write_str_raw(value);

    let mem = out.into_memory();

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

#[test]
fn test_write_str() {
    let out = OutStream::new();

    let value = "Hello World!";

    out.write_str(value);

    let mem = out.into_memory();

    assert_eq!(mem.len(), 1 + 4 + value.len());

    assert_eq!(mem[0], header::STRING as u8);

    assert_eq!(mem[1], value.len() as u8);
    assert_eq!(mem[2], 0);
    assert_eq!(mem[3], 0);
    assert_eq!(mem[4], 0);

    assert_eq!(mem[5], 'H' as u8);
    assert_eq!(mem[6], 'e' as u8);
    assert_eq!(mem[7], 'l' as u8);
    assert_eq!(mem[8], 'l' as u8);
    assert_eq!(mem[9], 'o' as u8);
    assert_eq!(mem[10], ' ' as u8);
    assert_eq!(mem[11], 'W' as u8);
    assert_eq!(mem[12], 'o' as u8);
    assert_eq!(mem[13], 'r' as u8);
    assert_eq!(mem[14], 'l' as u8);
    assert_eq!(mem[15], 'd' as u8);
    assert_eq!(mem[16], '!' as u8);
}

#[test]
fn test_reserve_i32() {
    let out = OutStream::new();

    let reserved = out.reserve_i32();

    out.write_i32(0x11223344);

    reserved.set(0x55667788);

    let mem = out.into_memory();

    assert_eq!(mem.len(), 8);

    assert_eq!(mem[0], 0x88);
    assert_eq!(mem[1], 0x77);
    assert_eq!(mem[2], 0x66);
    assert_eq!(mem[3], 0x55);

    assert_eq!(mem[4], 0x44);
    assert_eq!(mem[5], 0x33);
    assert_eq!(mem[6], 0x22);
    assert_eq!(mem[7], 0x11);
}

#[test]
#[should_panic]
fn test_reserve_i32_panic() {
    let out = OutStream::new();

    {
        let reserved = out.reserve_i32();
    }

    let mem = out.into_memory();
}

#[test]
fn test_reserve_len() {
    let out = OutStream::new();

    let reserved = out.reserve_len();

    out.write_i32(0x11223344);
    out.write_str_raw("Lorem ipsum");

    reserved.set();

    let mem = out.into_memory();

    assert_eq!(mem.len(), 4 + 4 + 4 + 11);

    assert_eq!(mem[0], 4 + 4 + 11);
    assert_eq!(mem[1], 0);
    assert_eq!(mem[2], 0);
    assert_eq!(mem[3], 0);

    assert_eq!(mem[4], 0x44);
    assert_eq!(mem[5], 0x33);
    assert_eq!(mem[6], 0x22);
    assert_eq!(mem[7], 0x11);
}
