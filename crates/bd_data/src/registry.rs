//! Generic Registry for content IDs.
//!
//! Maps ContentId → T with duplicate detection and reference checking.

use std::collections::HashMap;
use thiserror::Error;

use crate::id::ContentId;

/// Errors that can occur when working with Registries.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    #[error("Duplicate ID: {0}")]
    DuplicateId(ContentId),
    #[error("Missing ID: {0}")]
    MissingId(ContentId),
    #[error("Invalid ID format: {0}")]
    InvalidIdFormat(String),
    #[error("Wrong namespace: expected '{expected}', got '{got}'")]
    WrongNamespace { expected: String, got: String },
}

/// A generic content registry.
#[derive(Debug, Clone)]
pub struct Registry<T> {
    items: HashMap<ContentId, T>,
    namespace: Option<String>,
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
            namespace: None,
        }
    }
}

impl<T> Registry<T> {
    /// Create a new registry optionally scoped to a namespace.
    pub fn new(namespace: Option<String>) -> Self {
        Self {
            items: HashMap::new(),
            namespace,
        }
    }

    /// Insert a value. Errors on duplicate ID.
    pub fn insert(&mut self, id: ContentId, value: T) -> Result<(), RegistryError> {
        if let Some(ref ns) = self.namespace {
            if id.namespace != *ns {
                return Err(RegistryError::WrongNamespace {
                    expected: ns.clone(),
                    got: id.namespace,
                });
            }
        }
        if self.items.contains_key(&id) {
            return Err(RegistryError::DuplicateId(id));
        }
        self.items.insert(id, value);
        Ok(())
    }

    /// Get a value by ID.
    pub fn get(&self, id: &ContentId) -> Option<&T> {
        self.items.get(id)
    }

    /// Check if an ID exists.
    pub fn contains(&self, id: &ContentId) -> bool {
        self.items.contains_key(id)
    }

    /// Iterate over all (ID, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&ContentId, &T)> {
        self.items.iter()
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_inserts_and_retrieves() {
        let mut reg: Registry<i32> = Registry::default();
        let id = ContentId::new("test", "value");
        reg.insert(id.clone(), 42).unwrap();
        assert_eq!(*reg.get(&id).unwrap(), 42);
    }

    #[test]
    fn registry_rejects_duplicate_id() {
        let mut reg: Registry<i32> = Registry::default();
        let id = ContentId::new("test", "dup");
        reg.insert(id.clone(), 1).unwrap();
        let err = reg.insert(id.clone(), 2).unwrap_err();
        assert!(matches!(err, RegistryError::DuplicateId(_)));
    }

    #[test]
    fn registry_reports_missing_reference() {
        let reg: Registry<i32> = Registry::default();
        let id = ContentId::new("test", "missing");
        assert!(!reg.contains(&id));
        assert!(reg.get(&id).is_none());
    }

    #[test]
    fn registry_scoped_namespace() {
        let mut reg: Registry<i32> = Registry::new(Some("ability".into()));
        let wrong = ContentId::new("status", "poisoned");
        let err = reg.insert(wrong, 1).unwrap_err();
        assert!(matches!(err, RegistryError::WrongNamespace { .. }));
    }
}
