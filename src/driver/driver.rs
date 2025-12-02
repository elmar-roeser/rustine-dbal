//! Driver trait for database abstraction

use async_trait::async_trait;
use crate::core::{ConnectionParams, Result};

use super::DriverConnection;

/// A database driver that can create connections
#[async_trait]
pub trait Driver: Send + Sync {
    /// The connection type produced by this driver
    type Connection: DriverConnection;

    /// Create a new connection to the database
    async fn connect(&self, params: &ConnectionParams) -> Result<Self::Connection>;

    /// Get the name of this driver
    fn name(&self) -> &'static str;

    /// Check if this driver supports the given platform
    fn supports(&self, driver_name: &str) -> bool {
        self.name() == driver_name
    }
}
