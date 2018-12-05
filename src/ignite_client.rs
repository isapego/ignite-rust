use super::IgniteConfiguration;

#[derive(Debug)]
pub struct IgniteClient {}

impl IgniteClient {
    /// Start Ignite client with default configuration.
    pub fn start_default() -> Result<IgniteClient, String> {
        Ok(IgniteClient {})
    }

    pub fn start(_cfg: IgniteConfiguration) -> Result<IgniteClient, String> {
        Ok(IgniteClient {})
    }
}
