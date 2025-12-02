//! Configuration types for Rustine DBAL
//!
//! Provides connection parameters and runtime configuration.

use std::time::Duration;

/// Connection parameters for establishing database connections
#[derive(Debug, Clone)]
pub struct ConnectionParams {
    /// Database driver type (e.g., "postgres", "mysql", "sqlite")
    pub driver: String,

    /// Database host
    pub host: Option<String>,

    /// Database port
    pub port: Option<u16>,

    /// Database name
    pub database: Option<String>,

    /// Username for authentication
    pub username: Option<String>,

    /// Password for authentication
    pub password: Option<String>,

    /// Unix socket path (alternative to host/port)
    pub socket: Option<String>,

    /// Path to database file (for `SQLite`)
    pub path: Option<String>,

    /// Additional driver-specific options
    pub options: std::collections::HashMap<String, String>,
}

impl ConnectionParams {
    /// Create new connection parameters with driver type
    #[must_use]
    pub fn new(driver: impl Into<String>) -> Self {
        Self {
            driver: driver.into(),
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
            socket: None,
            path: None,
            options: std::collections::HashMap::new(),
        }
    }

    /// Create connection parameters for `PostgreSQL`
    #[must_use]
    pub fn postgres() -> Self {
        Self::new("postgres").with_port(5432)
    }

    /// Create connection parameters for `MySQL`
    #[must_use]
    pub fn mysql() -> Self {
        Self::new("mysql").with_port(3306)
    }

    /// Create connection parameters for `SQLite`
    #[must_use]
    pub fn sqlite() -> Self {
        Self::new("sqlite")
    }

    /// Create connection parameters for `SQLite` in-memory database
    #[must_use]
    pub fn sqlite_memory() -> Self {
        Self::new("sqlite").with_path(":memory:")
    }

    /// Set host
    #[must_use]
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Set port
    #[must_use]
    pub const fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set database name
    #[must_use]
    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    /// Set username
    #[must_use]
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Set password
    #[must_use]
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set Unix socket path
    #[must_use]
    pub fn with_socket(mut self, socket: impl Into<String>) -> Self {
        self.socket = Some(socket.into());
        self
    }

    /// Set file path (for `SQLite`)
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set a driver-specific option
    #[must_use]
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }

    /// Parse a connection URL into `ConnectionParams`
    ///
    /// Supported formats:
    /// - `postgres://user:pass@host:port/database`
    /// - `mysql://user:pass@host:port/database`
    /// - `sqlite:///path/to/database.db`
    /// - `sqlite::memory:` or `sqlite://:memory:`
    ///
    /// # Errors
    ///
    /// Returns a configuration error if the URL format is invalid.
    pub fn from_url(url: &str) -> crate::Result<Self> {
        // Handle SQLite special shorthand format "sqlite::memory:"
        if url == "sqlite::memory:" {
            return Ok(Self::new("sqlite").with_path(":memory:"));
        }

        // Simple URL parsing without external dependencies
        let (driver, rest) = url.split_once("://")
            .ok_or_else(|| crate::Error::config("Invalid URL: missing scheme"))?;

        let mut params = Self::new(driver);

        // Handle SQLite special cases
        if driver == "sqlite" {
            if rest == ":memory:" || rest.is_empty() {
                params.path = Some(":memory:".to_string());
            } else {
                // Remove leading slash if present (sqlite:///path -> /path)
                let path = rest.strip_prefix('/').unwrap_or(rest);
                params.path = Some(path.to_string());
            }
            return Ok(params);
        }

        // Parse user:pass@host:port/database
        let (auth_host, database) = if let Some((before, after)) = rest.rsplit_once('/') {
            (before, Some(after.to_string()))
        } else {
            (rest, None)
        };
        params.database = database;

        let (auth, host_port) = if let Some((before, after)) = auth_host.split_once('@') {
            (Some(before), after)
        } else {
            (None, auth_host)
        };

        // Parse auth (user:pass)
        if let Some(auth) = auth {
            let (user, pass) = if let Some((u, p)) = auth.split_once(':') {
                (Some(u.to_string()), Some(p.to_string()))
            } else {
                (Some(auth.to_string()), None)
            };
            params.username = user;
            params.password = pass;
        }

        // Parse host:port
        if let Some((host, port_str)) = host_port.rsplit_once(':') {
            params.host = Some(host.to_string());
            params.port = port_str.parse().ok();
        } else if !host_port.is_empty() {
            params.host = Some(host_port.to_string());
        }

        // Set default ports
        if params.port.is_none() {
            params.port = match driver {
                "postgres" | "postgresql" => Some(5432),
                "mysql" | "mariadb" => Some(3306),
                "mssql" | "sqlserver" => Some(1433),
                _ => None,
            };
        }

        Ok(params)
    }

    /// Convert to a connection URL string
    #[must_use]
    pub fn to_url(&self) -> String {
        let mut url = format!("{}://", self.driver);

        if self.driver == "sqlite" {
            if let Some(path) = &self.path {
                if path == ":memory:" {
                    url.push_str(":memory:");
                } else {
                    url.push('/');
                    url.push_str(path);
                }
            }
            return url;
        }

        if let Some(username) = &self.username {
            url.push_str(username);
            if let Some(password) = &self.password {
                url.push(':');
                url.push_str(password);
            }
            url.push('@');
        }

        if let Some(host) = &self.host {
            url.push_str(host);
        }

        if let Some(port) = self.port {
            url.push(':');
            url.push_str(&port.to_string());
        }

        if let Some(database) = &self.database {
            url.push('/');
            url.push_str(database);
        }

        url
    }
}

impl Default for ConnectionParams {
    fn default() -> Self {
        Self::new("sqlite").with_path(":memory:")
    }
}

/// Runtime configuration for connections
#[derive(Debug, Clone)]
pub struct Configuration {
    /// Whether to automatically commit after each statement (when not in a transaction)
    pub auto_commit: bool,

    /// Connection timeout
    pub connect_timeout: Option<Duration>,

    /// Query execution timeout
    pub query_timeout: Option<Duration>,

    /// Whether to use lazy connection (connect on first query)
    pub lazy_connect: bool,

    /// Schema/search path for the connection
    pub schema: Option<String>,

    /// Character set for the connection
    pub charset: Option<String>,

    /// Timezone for the connection
    pub timezone: Option<String>,

    /// Application name (sent to database for logging)
    pub application_name: Option<String>,

    /// Whether to enable query logging
    pub enable_logging: bool,

    /// Custom datetime format string
    pub datetime_format: Option<String>,

    /// Custom date format string
    pub date_format: Option<String>,

    /// Custom time format string
    pub time_format: Option<String>,
}

impl Configuration {
    /// Create a new configuration with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set auto-commit mode
    #[must_use]
    pub const fn with_auto_commit(mut self, auto_commit: bool) -> Self {
        self.auto_commit = auto_commit;
        self
    }

    /// Set connection timeout
    #[must_use]
    pub const fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Set query timeout
    #[must_use]
    pub const fn with_query_timeout(mut self, timeout: Duration) -> Self {
        self.query_timeout = Some(timeout);
        self
    }

    /// Set lazy connection mode
    #[must_use]
    pub const fn with_lazy_connect(mut self, lazy: bool) -> Self {
        self.lazy_connect = lazy;
        self
    }

    /// Set schema/search path
    #[must_use]
    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    /// Set character set
    #[must_use]
    pub fn with_charset(mut self, charset: impl Into<String>) -> Self {
        self.charset = Some(charset.into());
        self
    }

    /// Set timezone
    #[must_use]
    pub fn with_timezone(mut self, timezone: impl Into<String>) -> Self {
        self.timezone = Some(timezone.into());
        self
    }

    /// Set application name
    #[must_use]
    pub fn with_application_name(mut self, name: impl Into<String>) -> Self {
        self.application_name = Some(name.into());
        self
    }

    /// Enable query logging
    #[must_use]
    pub const fn with_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    /// Set custom datetime format
    #[must_use]
    pub fn with_datetime_format(mut self, format: impl Into<String>) -> Self {
        self.datetime_format = Some(format.into());
        self
    }

    /// Set custom date format
    #[must_use]
    pub fn with_date_format(mut self, format: impl Into<String>) -> Self {
        self.date_format = Some(format.into());
        self
    }

    /// Set custom time format
    #[must_use]
    pub fn with_time_format(mut self, format: impl Into<String>) -> Self {
        self.time_format = Some(format.into());
        self
    }

    /// Get datetime format (returns default if not set)
    #[must_use]
    pub fn datetime_format(&self) -> &str {
        self.datetime_format.as_deref().unwrap_or("%Y-%m-%d %H:%M:%S")
    }

    /// Get date format (returns default if not set)
    #[must_use]
    pub fn date_format(&self) -> &str {
        self.date_format.as_deref().unwrap_or("%Y-%m-%d")
    }

    /// Get time format (returns default if not set)
    #[must_use]
    pub fn time_format(&self) -> &str {
        self.time_format.as_deref().unwrap_or("%H:%M:%S")
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            auto_commit: true,
            connect_timeout: Some(Duration::from_secs(30)),
            query_timeout: None,
            lazy_connect: true,
            schema: None,
            charset: Some("utf8".to_string()),
            timezone: None,
            application_name: Some("rustine".to_string()),
            enable_logging: false,
            datetime_format: None,
            date_format: None,
            time_format: None,
        }
    }
}

/// Transaction isolation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationLevel {
    /// Read uncommitted - lowest isolation, allows dirty reads
    ReadUncommitted,

    /// Read committed - default for most databases
    #[default]
    ReadCommitted,

    /// Repeatable read - prevents non-repeatable reads
    RepeatableRead,

    /// Serializable - highest isolation, prevents all anomalies
    Serializable,
}

impl IsolationLevel {
    /// Get the SQL representation of this isolation level
    #[must_use]
    pub const fn as_sql(&self) -> &'static str {
        match self {
            Self::ReadUncommitted => "READ UNCOMMITTED",
            Self::ReadCommitted => "READ COMMITTED",
            Self::RepeatableRead => "REPEATABLE READ",
            Self::Serializable => "SERIALIZABLE",
        }
    }
}

impl std::fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_sql())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_params_postgres() {
        let params = ConnectionParams::postgres()
            .with_host("localhost")
            .with_database("mydb")
            .with_username("user")
            .with_password("pass");

        assert_eq!(params.driver, "postgres");
        assert_eq!(params.host, Some("localhost".to_string()));
        assert_eq!(params.port, Some(5432));
        assert_eq!(params.database, Some("mydb".to_string()));
        assert_eq!(params.username, Some("user".to_string()));
        assert_eq!(params.password, Some("pass".to_string()));
    }

    #[test]
    fn test_connection_params_from_url() {
        let params = ConnectionParams::from_url("postgres://user:pass@localhost:5432/mydb").unwrap();
        assert_eq!(params.driver, "postgres");
        assert_eq!(params.host, Some("localhost".to_string()));
        assert_eq!(params.port, Some(5432));
        assert_eq!(params.database, Some("mydb".to_string()));
        assert_eq!(params.username, Some("user".to_string()));
        assert_eq!(params.password, Some("pass".to_string()));
    }

    #[test]
    fn test_connection_params_sqlite_memory() {
        let params = ConnectionParams::from_url("sqlite::memory:").unwrap();
        assert_eq!(params.driver, "sqlite");
        assert_eq!(params.path, Some(":memory:".to_string()));
    }

    #[test]
    fn test_connection_params_sqlite_file() {
        let params = ConnectionParams::from_url("sqlite:///path/to/db.sqlite").unwrap();
        assert_eq!(params.driver, "sqlite");
        assert_eq!(params.path, Some("path/to/db.sqlite".to_string()));
    }

    #[test]
    fn test_connection_params_to_url() {
        let params = ConnectionParams::postgres()
            .with_host("localhost")
            .with_database("mydb")
            .with_username("user")
            .with_password("pass");

        assert_eq!(params.to_url(), "postgres://user:pass@localhost:5432/mydb");
    }

    #[test]
    fn test_configuration_defaults() {
        let config = Configuration::default();
        assert!(config.auto_commit);
        assert!(config.lazy_connect);
        assert_eq!(config.charset, Some("utf8".to_string()));
    }

    #[test]
    fn test_configuration_builder() {
        let config = Configuration::new()
            .with_auto_commit(false)
            .with_connect_timeout(Duration::from_secs(10))
            .with_schema("public");

        assert!(!config.auto_commit);
        assert_eq!(config.connect_timeout, Some(Duration::from_secs(10)));
        assert_eq!(config.schema, Some("public".to_string()));
    }

    #[test]
    fn test_isolation_level() {
        assert_eq!(IsolationLevel::ReadCommitted.as_sql(), "READ COMMITTED");
        assert_eq!(IsolationLevel::Serializable.as_sql(), "SERIALIZABLE");
        assert_eq!(IsolationLevel::default(), IsolationLevel::ReadCommitted);
    }
}
