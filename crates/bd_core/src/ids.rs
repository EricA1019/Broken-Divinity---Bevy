//! Content IDs — validated resource identifiers.
//!
//! Lives in bd_core so both bd_core (ActionRegistry) and bd_data (data loading)
//! can use it without circular dependencies.

use std::{fmt, str::FromStr};

use thiserror::Error;

/// A validated content identifier in `namespace.name` format.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentId {
    pub namespace: String,
    pub name: String,
}

impl ContentId {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
        }
    }

    /// Parse a "namespace.name" string with validation.
    pub fn parse(s: &str) -> Result<Self, ContentIdError> {
        if s.contains(' ') {
            return Err(ContentIdError::HasSpaces(s.into()));
        }
        if s.chars().any(|c| c.is_uppercase()) {
            return Err(ContentIdError::HasUppercase(s.into()));
        }
        let dot = s
            .find('.')
            .ok_or_else(|| ContentIdError::InvalidFormat(s.into()))?;
        let namespace = &s[..dot];
        let name = &s[dot + 1..];
        if namespace.is_empty() || name.is_empty() {
            return Err(ContentIdError::InvalidFormat(s.into()));
        }
        Ok(Self {
            namespace: namespace.into(),
            name: name.into(),
        })
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.namespace, self.name)
    }
}

impl FromStr for ContentId {
    type Err = ContentIdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ContentIdError {
    #[error("Invalid ContentId format: {0}. Expected 'namespace.name'")]
    InvalidFormat(String),
    #[error("ContentId '{0}' contains uppercase characters")]
    HasUppercase(String),
    #[error("ContentId '{0}' contains spaces")]
    HasSpaces(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_id_parses_valid_id() {
        let id = ContentId::parse("ability.move").unwrap();
        assert_eq!(id.namespace, "ability");
        assert_eq!(id.name, "move");
        assert_eq!(id.to_string(), "ability.move");
    }

    #[test]
    fn content_id_rejects_space() {
        assert!(ContentId::parse("ability. move").is_err());
    }

    #[test]
    fn content_id_rejects_uppercase() {
        assert!(ContentId::parse("Ability.Move").is_err());
    }

    #[test]
    fn content_id_rejects_no_dot() {
        assert!(ContentId::parse("invalid").is_err());
    }
}
