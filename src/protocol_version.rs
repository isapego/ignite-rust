use crate::protocol::{InStream, OutStream, Readable, Writable};

/// Version 1.2.0
pub const VERSION_1_2_0: ProtocolVersion = ProtocolVersion {
    major: 1,
    minor: 2,
    maintenance: 0,
};

#[derive(Copy, Clone, Debug)]
pub struct ProtocolVersion {
    major: i16,
    minor: i16,
    maintenance: i16,
}

impl ProtocolVersion {
    /// Make new instance
    pub fn new(major: i16, minor: i16, maintenance: i16) -> Self {
        Self {
            major,
            minor,
            maintenance,
        }
    }
}

impl Writable for ProtocolVersion {
    fn write(&self, out: &OutStream) {
        out.write_i16(self.major);
        out.write_i16(self.minor);
        out.write_i16(self.maintenance);
    }
}

impl Readable for ProtocolVersion {
    type Item = Self;

    fn read(stream: &InStream) -> Self {
        let major = stream.read_i16();
        let minor = stream.read_i16();
        let maintenance = stream.read_i16();

        Self {
            major,
            minor,
            maintenance,
        }
    }
}
