use std::convert::Into;

#[derive(Debug)]
pub struct IgniteConfiguration {
    end_points :String
}

impl IgniteConfiguration {
    /// Create new configuration with default parameters.
    pub fn new() -> IgniteConfiguration {
        IgniteConfiguration{ end_points : String::new() }
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
    pub fn set_endpoints<S: Into<String>>(&mut self, end_points: S) {
        self.end_points = end_points.into();
    }

    /// Get endpoints.
    /// See set_endpoints() for details on the format.
    pub fn get_endpoints(&self) -> &str {
        self.end_points.as_str()
    }
}

#[test]
fn ignite_configuration_new() {
    IgniteConfiguration::new();
}

#[test]
fn ignite_configuration_endpoints() {
    let mut cfg = IgniteConfiguration::new();

    let end_points = "127.0.0.1:10800";

    cfg.set_endpoints(end_points);
    assert_eq!(cfg.get_endpoints(), end_points);
}