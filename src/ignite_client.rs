#[derive(Debug)]
struct IgniteClient {
}

impl IgniteClient {
    /// Start Ignite client with default configuration.
    pub fn start() -> Result<IgniteClient, String> {
        Ok(IgniteClient{})
    }
}

#[test]
fn ignite_client_new_default() {
    IgniteClient::start().unwrap();
}