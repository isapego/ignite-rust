use crate::net::utils;
use crate::net::EndPoint;
use crate::IgniteResult;
use std::convert::Into;

#[derive(Debug)]
pub struct IgniteConfiguration {
    end_points: Vec<EndPoint>,
    user: String,
    pass: String,
}

impl IgniteConfiguration {
    /// Create new configuration with default parameters.
    pub fn new() -> IgniteConfiguration {
        IgniteConfiguration {
            end_points: Vec::new(),
            user: String::new(),
            pass: String::new(),
        }
    }

    /// Set endpoints to connect to.
    ///
    /// The format is `"<host>[:<port>[..<port>]][,...]"`.
    ///
    /// IgniteClient is going to try connecting to hosts in random order checking all ports
    /// subsequently in port range for evey host. If port is not specified then the default
    /// port is used.
    ///
    /// # Examples
    /// ```
    /// use ignite_rust::IgniteConfiguration;
    ///
    /// let mut cfg = IgniteConfiguration::new();
    /// cfg.set_endpoints("127.0.0.1");
    /// cfg.set_endpoints("127.0.0.1:10800");
    /// cfg.set_endpoints("example.com");
    /// cfg.set_endpoints("127.0.0.1,example:1234..1500");
    /// ```
    pub fn set_endpoints(&mut self, end_points: &str) -> IgniteResult<()> {
        self.end_points = utils::parse_endpoints(end_points.into())?;
        Ok(())
    }

    /// Get endpoints.
    /// See set_endpoints() for details on the format.
    pub fn get_endpoints(&self) -> &[EndPoint] {
        &self.end_points
    }

    /// Get username for authentication
    pub fn get_user(&self) -> &str {
        &self.user
    }

    /// Get password for authentication
    pub fn get_password(&self) -> &str {
        &self.pass
    }
}

#[test]
fn ignite_configuration_new() {
    IgniteConfiguration::new();
}

#[test]
fn ignite_configuration_endpoints() {
    let mut cfg = IgniteConfiguration::new();

    let set = "127.0.0.1:10800";

    cfg.set_endpoints(set).unwrap();
    let get = cfg.get_endpoints();

    assert_eq!(get.len(), 1);
    assert_eq!(get[0], EndPoint::from_string(set).unwrap());
}
