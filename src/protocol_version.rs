use crate::protocol::{InStream, OutStream, Readable, Writable};

#[derive(Copy, Clone, Debug)]
pub struct ProtocolVersion {
    major: i16,
    minor: i16,
    maintaince: i16,
}

impl Writable for ProtocolVersion {
    fn write(&self, out: &OutStream) {
        out.write_i16(self.major);
        out.write_i16(self.minor);
        out.write_i16(self.maintaince);
    }
}

// impl Readable for ProtocolVersion {
//     type Item = Self;

//     fn read(stream: &InStream) -> Self {
//         let major = stream.read_i16();
//         let minor = stream.read_i16();
//         let maintaince = stream.read_i16();

//         Self{major: major, minor: minor, maintaince: maintaince}
//     }
// }
