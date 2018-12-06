extern crate ignite_rust;
extern crate rand;

use ignite_rust::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

fn make_unique_name() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(64).collect()
}

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

#[test]
fn ignite_client_create_cache() {
    let mut cfg = IgniteConfiguration::new();
    cfg.set_endpoints("127.0.0.1:10800");

    let mut _client = IgniteClient::start(cfg).unwrap();

    let _cache_name = make_unique_name();

    // client.create_cache(cache_name).expect("Success expected");
    // client.create_cache(cache_name).expect_err("Error expected: cache with the name should be created already");
}