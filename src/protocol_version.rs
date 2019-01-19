use protocol::{OutStream, Writable};

pub struct ProtocolVersion {
    major: i16,
    minor: i16,
    maintaince: i16,
}

impl Writable for ProtocolVersion {
    fn write(&self, out: &mut OutStream) {
        out.write_i16(self.major);
        out.write_i16(self.minor);
        out.write_i16(self.maintaince);
    }
}
