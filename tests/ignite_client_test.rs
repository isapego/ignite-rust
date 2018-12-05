extern crate ignite_rust;

use ignite_rust::*;

#[test]
fn ignite_client_start_default() {
    IgniteClient::start_default().unwrap();
}

#[test]
fn ignite_client_start_with_config() {
    let mut cfg = IgniteConfiguration::new();
    cfg.set_endpoints("127.0.0.1:10800");

    IgniteClient::start(cfg).unwrap();
}
