use std::convert::Into;
use std::iter::{IntoIterator, Iterator};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use crate::ignite_error::{RewrapResult, IgniteError, IgniteResult};

pub const DEFAULT_PORT: u16 = 10800;

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

    // Get host.
    pub fn host(&self) -> &str {
        self.host.as_ref()
    }

    /// Convert from string.
    /// The format is `"<host>[:<port>[..<port>]][,...]"`.
    pub fn from_string<'a, S: Into<&'a str>>(sadr: S) -> IgniteResult<Self> {
        let mut iter = sadr.into().trim().split(':');

        let host = match iter.next() {
            Some(h) => h,
            None => {
                return Err(IgniteError::new(
                    "Parsing error: Host can not be an empty string",
                ))
            }
        };

        if host.is_empty() {
            return Err(IgniteError::new(
                "Parsing error: Host can not be an empty string",
            ));
        }

        let range = match iter.next() {
            Some(r) => r,
            None => return Ok(EndPoint::new(host, DEFAULT_PORT, 0)),
        };

        if iter.next().is_some() {
            return Err(IgniteError::new(
                "Parsing error: Unexpected number of semicolons ':' in endpoint",
            ));
        };

        let mut range_iter = range.trim().split("..");

        let port_begin_s = match range_iter.next() {
            Some(p) => p,
            None => {
                return Err(IgniteError::new(
                    "Parsing error: Port can not be an empty string",
                ))
            }
        };

        let port_begin = port_begin_s
            .parse::<u16>()
            .rewrap_on_error("Parsing error: can not parse port")?;

        if port_begin == 0 {
            return Err(IgniteError::new("Parsing error: TCP port can not be zero"));
        }

        let port_end_s = match range_iter.next() {
            Some(p) => p,
            None => return Ok(EndPoint::new(host, port_begin, 0)),
        };

        let port_end = port_end_s
            .parse::<u16>()
            .rewrap_on_error("Parsing error: can not parse port range")?;

        if port_begin > port_end {
            return Err(IgniteError::new(
                "Parsing error: beginning of the port range can not be bigger than the end",
            ));
        }

        if range_iter.next().is_some() {
            return Err(IgniteError::new(
                "Parsing error: Unexpected number of range separators '..' in endpoint",
            ));
        };

        Ok(EndPoint::new(host, port_begin, port_end))
    }

    /// Resolve host IPs
    pub fn resolve(&self) -> IgniteResult<ResolvedEndPoint> {
        let addr_tuple = (self.host.as_str(), self.port_begin);
        let iter = addr_tuple
            .to_socket_addrs()
            .rewrap_on_error(format!("Failed to resolve host address: {}", self.host))?;

        let ips = iter.map(|addr| addr.ip()).collect();
        Ok(ResolvedEndPoint {
            ips: ips,
            port_begin: self.port_begin,
            port_end: self.port_end,
        })
    }
}

/// Endpoint, pointing to a single host with a possible range of TCP ports.
#[derive(Debug)]
pub struct ResolvedEndPoint {
    ips: Vec<IpAddr>,
    port_begin: u16,
    port_end: u16,
}

/// Iterates over all possible addresses for the endpoint
#[derive(Debug)]
pub struct EndPointIterator {
    ips: Vec<IpAddr>,
    ip_idx: usize,
    port: u16,
    end_port: u16,
}

impl EndPointIterator {
    /// Create new instance
    fn new(end_point: ResolvedEndPoint) -> Self {
        EndPointIterator {
            ips: end_point.ips,
            ip_idx: 0,
            port: end_point.port_begin,
            end_port: end_point.port_end,
        }
    }
}

impl Iterator for EndPointIterator {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ip_idx < self.ips.len() {
            self.ip_idx += 1;
            return Some(SocketAddr::new(self.ips[self.ip_idx - 1], self.port));
        }

        if self.port < self.end_port {
            self.port += 1;
            self.ip_idx = 0;
            return Some(SocketAddr::new(self.ips[self.ip_idx], self.port));
        }

        None
    }
}

impl IntoIterator for ResolvedEndPoint {
    type Item = SocketAddr;
    type IntoIter = EndPointIterator;

    fn into_iter(self) -> Self::IntoIter {
        EndPointIterator::new(self)
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
