mod ignite_client;
mod ignite_configuration;
mod ignite_error;

pub use ignite_error::IgniteError;
pub use ignite_configuration::IgniteConfiguration;
pub use ignite_client::IgniteClient;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
