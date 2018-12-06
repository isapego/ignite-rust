use std::rc::Rc;

use super::ignite_configuration::IgniteConfiguration;
use super::ignite_error::IgniteResult;

/// Component which is responsible for establishing and
/// maintaining reliable connection link to the Ignite cluster.
///
/// It aslo responsible for choosing which connection to use for
/// a certain request.
#[derive(Debug)]
pub struct DataRouter {
    cfg: Rc<IgniteConfiguration>,
}

impl DataRouter {
    /// Make new instance
    pub fn new(cfg: Rc<IgniteConfiguration>) -> DataRouter {
        DataRouter { cfg: cfg }
    }

    /// Try establish initial connection with Ignite cluster
    pub fn initial_connect(&mut self) -> IgniteResult<()> {
        Ok(())
    }
}
