use super::ignite_error::{IgniteError, IgniteResult};
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

        let range = match iter.next() {
            Some(r) => r,
            None => return Ok(EndPoint::new(host, 0, 0)),
        };

        if iter.next().is_some() {
            return Err("Parsing error: Unexpected number of semicolons in endpoint".into());
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

        Ok(EndPoint::new(host, port_begin, port_end))
    }
}

// Parse endpoints string.
// The format is `"<host>[:<port>[..<port>]][,...]"`.
// pub fn parse_endpoints(end_points: &str) -> SocketAddr {
//     end_points.split(',').map(|end_point| {

//     }).collect()
// }
