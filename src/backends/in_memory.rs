use crate::StorageBackend;
use crate::{PersistentError, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, hash::Hash};

/// An in-memory backend that doesn't persist data.
///
/// This backend is useful for testing or when persistence is not needed.
#[derive(Debug, Default)]
pub struct InMemoryBackend;

impl InMemoryBackend {
    /// Creates a new in-memory backend.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use persistent_map::in_memory::InMemoryBackend;
    ///
    /// let backend = InMemoryBackend::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl<K, V> StorageBackend<K, V> for InMemoryBackend
where
    K: Eq + Hash + Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    async fn load_all(&self) -> Result<HashMap<K, V>, PersistentError> {
        Ok(HashMap::new())
    }

    async fn save(&self, _key: K, _value: V) -> Result<(), PersistentError> {
        Ok(())
    }

    async fn delete(&self, _key: &K) -> Result<(), PersistentError> {
        Ok(())
    }
}
