//! Parameter types for prepared statement binding
//!
//! Defines the types used when binding parameters to prepared statements.

/// Parameter binding type for prepared statements
///
/// This enum indicates how a parameter should be bound to a prepared statement.
/// Different database drivers may handle these types differently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ParameterType {
    /// Null value
    Null,

    /// Integer value (i32, i64, etc.)
    Integer,

    /// String value
    #[default]
    String,

    /// Large object / binary data
    LargeObject,

    /// Boolean value
    Boolean,

    /// Binary data (BLOB)
    Binary,

    /// ASCII-only string (for optimization on some platforms)
    Ascii,
}

impl ParameterType {
    /// Check if this parameter type represents a null value
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Check if this parameter type represents binary data
    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Binary | Self::LargeObject)
    }

    /// Check if this parameter type represents text data
    pub fn is_text(&self) -> bool {
        matches!(self, Self::String | Self::Ascii)
    }
}

impl std::fmt::Display for ParameterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "NULL"),
            Self::Integer => write!(f, "INTEGER"),
            Self::String => write!(f, "STRING"),
            Self::LargeObject => write!(f, "LOB"),
            Self::Boolean => write!(f, "BOOLEAN"),
            Self::Binary => write!(f, "BINARY"),
            Self::Ascii => write!(f, "ASCII"),
        }
    }
}

/// A named or positional parameter with its type
#[derive(Debug, Clone)]
pub enum Parameter {
    /// Positional parameter (e.g., $1, ?)
    Positional {
        index: usize,
        param_type: ParameterType,
    },
    /// Named parameter (e.g., :name)
    Named {
        name: String,
        param_type: ParameterType,
    },
}

impl Parameter {
    /// Create a new positional parameter
    pub fn positional(index: usize, param_type: ParameterType) -> Self {
        Self::Positional { index, param_type }
    }

    /// Create a new named parameter
    pub fn named(name: impl Into<String>, param_type: ParameterType) -> Self {
        Self::Named {
            name: name.into(),
            param_type,
        }
    }

    /// Get the parameter type
    pub fn param_type(&self) -> ParameterType {
        match self {
            Self::Positional { param_type, .. } => *param_type,
            Self::Named { param_type, .. } => *param_type,
        }
    }

    /// Check if this is a positional parameter
    pub fn is_positional(&self) -> bool {
        matches!(self, Self::Positional { .. })
    }

    /// Check if this is a named parameter
    pub fn is_named(&self) -> bool {
        matches!(self, Self::Named { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_type_default() {
        assert_eq!(ParameterType::default(), ParameterType::String);
    }

    #[test]
    fn test_parameter_type_display() {
        assert_eq!(ParameterType::Integer.to_string(), "INTEGER");
        assert_eq!(ParameterType::String.to_string(), "STRING");
        assert_eq!(ParameterType::Boolean.to_string(), "BOOLEAN");
    }

    #[test]
    fn test_parameter_type_checks() {
        assert!(ParameterType::Null.is_null());
        assert!(!ParameterType::String.is_null());

        assert!(ParameterType::Binary.is_binary());
        assert!(ParameterType::LargeObject.is_binary());
        assert!(!ParameterType::String.is_binary());

        assert!(ParameterType::String.is_text());
        assert!(ParameterType::Ascii.is_text());
        assert!(!ParameterType::Integer.is_text());
    }

    #[test]
    fn test_parameter_creation() {
        let pos = Parameter::positional(0, ParameterType::Integer);
        assert!(pos.is_positional());
        assert_eq!(pos.param_type(), ParameterType::Integer);

        let named = Parameter::named("user_id", ParameterType::Integer);
        assert!(named.is_named());
        assert_eq!(named.param_type(), ParameterType::Integer);
    }
}
