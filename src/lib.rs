
mod ignite_client;
mod ignite_configuration;

pub use ignite_configuration::IgniteConfiguration;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
