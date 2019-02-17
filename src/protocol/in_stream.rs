use std::borrow::Cow;
use std::cell::Cell;

// Trait for a type that can be read from a stream
pub trait Readable {
    type Item: Sized;

    fn read(stream: &InStream) -> Self::Item;
}

pub struct InStream<'a> {
    mem: &'a [u8],
    pos: Cell<usize>,
}

impl<'a> InStream<'a> {
    /// Make new instance
    pub fn new(mem: &'a [u8]) -> Self {
        Self {
            mem,
            pos: Cell::new(0),
        }
    }

    /// Read bool value from the stream
    pub fn read_bool(&self) -> bool {
        self.read_i8() != 0
    }

    /// Read i16 value from the stream
    pub fn read_i8(&self) -> i8 {
        let pos = self.pos.get();

        self.inc_pos(1);

        self.mem[pos] as i8
    }

    /// Read i16 value from the stream
    pub fn read_i16(&self) -> i16 {
        let pos = self.pos.get();

        self.inc_pos(2);

        (self.mem[pos] as i16 & 0xFFi16) | ((self.mem[pos + 1] as i16 & 0xFFi16) << 8)
    }

    /// Read i32 value from the stream
    pub fn read_i32(&self) -> i32 {
        let pos = self.pos.get();

        self.inc_pos(4);

        (self.mem[pos] as i32 & 0xFFi32)
            | ((self.mem[pos + 1] as i32 & 0xFFi32) << 8)
            | ((self.mem[pos + 2] as i32 & 0xFFi32) << 16)
            | ((self.mem[pos + 3] as i32 & 0xFFi32) << 24)
    }

    /// Read i64 value from the stream
    pub fn read_i64(&self) -> i64 {
        let pos = self.pos.get();

        self.inc_pos(8);

        (self.mem[pos] as i64 & 0xFFi64)
            | ((self.mem[pos + 1] as i64 & 0xFFi64) << 8)
            | ((self.mem[pos + 2] as i64 & 0xFFi64) << 16)
            | ((self.mem[pos + 3] as i64 & 0xFFi64) << 24)
            | ((self.mem[pos + 4] as i64 & 0xFFi64) << 32)
            | ((self.mem[pos + 5] as i64 & 0xFFi64) << 40)
            | ((self.mem[pos + 6] as i64 & 0xFFi64) << 48)
            | ((self.mem[pos + 7] as i64 & 0xFFi64) << 56)
    }

    /// Read string
    pub fn read_str(&self) -> Cow<'a, str> {
        let len = self.read_i32();

        let pos = self.pos.get();

        self.inc_pos(4 + len as usize);

        String::from_utf8_lossy(&self.mem[pos..pos + len as usize])
    }

    /// Advance position for the specified value
    fn inc_pos(&self, val: usize) {
        self.pos.set(self.pos.get() + val);
    }
}
