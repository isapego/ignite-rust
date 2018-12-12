use ignite_error::{IgniteError, IgniteResult};
use std::convert::Into;
use std::iter::Iterator;

/// Endpoint, pointing to a single host with a possible range of TCP ports.
#[derive(Debug)]
pub struct EndPoint {
    host: String,
    port_begin: u16,
    port_end: u16,
}

impl EndPoint {
    /// Make new instance of the endpoint.
    fn new<S: Into<String>>(host: S, port_begin: u16, port_end: u16) -> Self {
        EndPoint {
            host: host.into(),
            port_begin: port_begin,
            port_end: port_end,
        }
    }

    /// Convert from string.
    /// The format is `"<host>[:<port>[..<port>]][,...]"`.
    pub fn from_string<'a, S: Into<&'a str>>(sadr: S) -> IgniteResult<Self> {
        let mut iter = sadr.into().trim().split(':');

        let host = match iter.next() {
            Some(h) => h,
            None => return Err("Parsing error: Host can not be an empty string".into()),
        };

        if host.is_empty() {
            return Err("Parsing error: Host can not be an empty string".into());
        }

        let range = match iter.next() {
            Some(r) => r,
            None => return Ok(EndPoint::new(host, 0, 0)),
        };

        if iter.next().is_some() {
            return Err("Parsing error: Unexpected number of semicolons ':' in endpoint".into());
        };

        let mut range_iter = range.trim().split("..");

        let port_begin_s = match range_iter.next() {
            Some(p) => p,
            None => return Err("Parsing error: Port can not be an empty string".into()),
        };

        let port_begin = match port_begin_s.parse::<u16>() {
            Ok(p) => p,
            Err(e) => {
                return Err(IgniteError::new_with_source(
                    "Parsing error: can not parse port",
                    Box::new(e),
                ))
            }
        };

        if port_begin == 0 {
            return Err("Parsing error: TCP port can not be zero".into());
        }

        let port_end_s = match range_iter.next() {
            Some(p) => p,
            None => return Ok(EndPoint::new(host, port_begin, 0)),
        };

        let port_end = match port_end_s.parse::<u16>() {
            Ok(p) => p,
            Err(e) => {
                return Err(IgniteError::new_with_source(
                    "Parsing error: can not parse port",
                    Box::new(e),
                ))
            }
        };

        if port_begin > port_end {
            return Err(
                "Parsing error: beginning of the port range can not be bigger than the end".into(),
            );
        }

        if range_iter.next().is_some() {
            return Err(
                "Parsing error: Unexpected number of range separators '..' in endpoint".into(),
            );
        };

        Ok(EndPoint::new(host, port_begin, port_end))
    }
}

#[test]
fn end_point_from_string() {
    EndPoint::from_string("127.0.0.1").unwrap();
    EndPoint::from_string("127.0.0.1:42").unwrap();
    EndPoint::from_string("127.0.0.1:1..2").unwrap();
    EndPoint::from_string("example.com").unwrap();
    EndPoint::from_string("example.com:42").unwrap();
    EndPoint::from_string("example.com:42..42").unwrap();
    EndPoint::from_string("example.com:42..101").unwrap();
    EndPoint::from_string("example.com:54..54").unwrap();

    EndPoint::from_string("").unwrap_err();
    EndPoint::from_string(":").unwrap_err();
    EndPoint::from_string(":0").unwrap_err();
    EndPoint::from_string(":0..12").unwrap_err();
    EndPoint::from_string(":12").unwrap_err();
    EndPoint::from_string(":41..42").unwrap_err();
    EndPoint::from_string(":42..41").unwrap_err();
    EndPoint::from_string(":..").unwrap_err();
    EndPoint::from_string(":42..").unwrap_err();
    EndPoint::from_string(":..42").unwrap_err();
    EndPoint::from_string(":..42..").unwrap_err();
    EndPoint::from_string(":12..42..").unwrap_err();
    EndPoint::from_string(":..12..42").unwrap_err();
    EndPoint::from_string(":..12..42..").unwrap_err();
    EndPoint::from_string(":47293875987").unwrap_err();
    EndPoint::from_string(":12234546..42341245436").unwrap_err();
    EndPoint::from_string(":1..47293875987").unwrap_err();

    EndPoint::from_string("127.0.0.1:").unwrap_err();
    EndPoint::from_string("127.0.0.1:0").unwrap_err();
    EndPoint::from_string("127.0.0.1:0..12").unwrap_err();
    EndPoint::from_string("127.0.0.1:42..41").unwrap_err();
    EndPoint::from_string("127.0.0.1:..").unwrap_err();
    EndPoint::from_string("127.0.0.1:42..").unwrap_err();
    EndPoint::from_string("127.0.0.1:..42").unwrap_err();
    EndPoint::from_string("127.0.0.1:..42..").unwrap_err();
    EndPoint::from_string("127.0.0.1:12..42..").unwrap_err();
    EndPoint::from_string("127.0.0.1:..12..42").unwrap_err();
    EndPoint::from_string("127.0.0.1:..12..42..").unwrap_err();
    EndPoint::from_string("127.0.0.1:47293875987").unwrap_err();
    EndPoint::from_string("127.0.0.1:12234546..42341245436").unwrap_err();
    EndPoint::from_string("127.0.0.1:1..47293875987").unwrap_err();

    EndPoint::from_string("example.com:").unwrap_err();
    EndPoint::from_string("example.com:0").unwrap_err();
    EndPoint::from_string("example.com:0..12").unwrap_err();
    EndPoint::from_string("example.com:42..41").unwrap_err();
    EndPoint::from_string("example.com:..").unwrap_err();
    EndPoint::from_string("example.com:42..").unwrap_err();
    EndPoint::from_string("example.com:..42").unwrap_err();
    EndPoint::from_string("example.com:..42..").unwrap_err();
    EndPoint::from_string("example.com:12..42..").unwrap_err();
    EndPoint::from_string("example.com:..12..42").unwrap_err();
    EndPoint::from_string("example.com:..12..42..").unwrap_err();
    EndPoint::from_string("example.com:47293875987").unwrap_err();
    EndPoint::from_string("example.com:12234546..42341245436").unwrap_err();
    EndPoint::from_string("example.com:1..47293875987").unwrap_err();
}
