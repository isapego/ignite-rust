#[derive(Debug)]
pub struct IgniteConfiguraton {
}

impl IgniteConfiguraton {
    fn new() -> IgniteConfiguraton {
        IgniteConfiguraton{}
    }
}

#[test]
fn ignite_configuration_new() {
    IgniteConfiguraton::new();
}