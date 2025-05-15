//! # `PersistentMap`
//!
//! `persistent-map` provides an in-memory key-value store with async, pluggable persistence.
//!
//! It combines the speed of [`DashMap`](https://docs.rs/dashmap) for in-memory operations
//! with the durability of various storage backends for persistence.
//!
//! ## Features
//!
//! - **Fast in-memory access**: Uses `DashMap` for concurrent read/write operations
//! - **Async API**: Non-blocking I/O operations for persistence
//! - **Multiple backends**: `SQLite`, `CSV`, in-memory, and extensible for more
//! - **Type-safe**: Works with any types that implement the required traits
//!
//! ## Example
//!
//! ```rust,no_run
//! use persistent_map::{PersistentMap, Result};
//! # #[cfg(feature = "sqlite")]
//! use persistent_map::sqlite::SqliteBackend;
//!
//! # #[cfg(feature = "sqlite")]
//! # async fn example() -> Result<()> {
//! # // Create a SQLite backend
//! # let backend = SqliteBackend::new("my_database.db").await?;
//! #
//! # // Initialize the persistent map
//! # let map = PersistentMap::new(backend).await?;
//! #
//! # // Insert a key-value pair (persists automatically)
//! # map.insert("hello".to_string(), "world".to_string()).await?;
//! #
//! # // Retrieve a value (from memory)
//! # assert_eq!(map.get(&"hello".to_string()), Some("world".to_string()));
//! # Ok(())
//! # }
//! #
//! # #[cfg(not(feature = "sqlite"))]
//! # fn example() {}
//! ```

use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, hash::Hash};
use thiserror::Error;
/// A trait for implementing storage backends for `PersistentMap`.
///
/// This trait defines the interface that all storage backends must implement.
/// It provides methods for loading, saving, and deleting key-value pairs.
///
/// # Type Parameters
///
/// * `K`: The key type, which must be hashable, serializable, and cloneable
/// * `V`: The value type, which must be serializable and cloneable
///
/// # Examples
///
/// Implementing a custom backend:
///
/// The `StorageBackend` trait defines the interface for persistent storage backends.
/// By implementing this trait, you can create custom storage solutions for the `PersistentMap`.
///
/// # Implementing a Custom Backend
///
/// To implement a custom backend, you need to:
///
/// 1. Create a struct to hold your backend-specific data
/// 2. Implement the required methods: `load_all`, `save`, and `delete`
/// 3. Optionally override the `flush` method if your backend buffers writes
///
/// # Example Implementation
///
/// Here's an example of a custom backend that stores data in a JSON file:
///
/// ```rust
/// use persistent_map::{StorageBackend, PersistentError, Result};
/// use std::collections::HashMap;
/// use std::path::PathBuf;
/// use std::fs;
/// use serde::{Serialize, de::DeserializeOwned};
/// use std::hash::Hash;
///
/// struct JsonFileBackend {
///     path: PathBuf,
/// }
///
/// impl JsonFileBackend {
///     pub fn new(path: impl Into<PathBuf>) -> Self {
///         Self { path: path.into() }
///     }
///
///     // Helper method to ensure the file exists
///     fn ensure_file_exists(&self) -> std::io::Result<()> {
///         if !self.path.exists() {
///             // Create parent directories if they don't exist
///             if let Some(parent) = self.path.parent() {
///                 if !parent.exists() {
///                     fs::create_dir_all(parent)?;
///                 }
///             }
///
///             // Create the file with an empty JSON object
///             fs::write(&self.path, "{}")?;
///         }
///         Ok(())
///     }
/// }
///
/// #[async_trait::async_trait]
/// impl<K, V> StorageBackend<K, V> for JsonFileBackend
/// where
///     K: Eq + Hash + Clone + Serialize + DeserializeOwned + Send + Sync + ToString + 'static,
///     V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
/// {
///     async fn load_all(&self) -> Result<HashMap<K, V>, PersistentError> {
///         // Ensure the file exists
///         self.ensure_file_exists()?;
///
///         // If the file is empty or contains just "{}", return an empty HashMap
///         let content = fs::read_to_string(&self.path)?;
///         if content.trim() == "{}" {
///             return Ok(HashMap::new());
///         }
///
///         // Parse the JSON file
///         let map = serde_json::from_str(&content)
///             .map_err(|e| PersistentError::Serde(e))?;
///
///         Ok(map)
///     }
///
///     async fn save(&self, key: K, value: V) -> Result<(), PersistentError> {
///         // Ensure the file exists
///         self.ensure_file_exists()?;
///
///         // Load existing data
///         let mut map: HashMap<K, V> = self.load_all().await?;
///
///         // Update the map
///         map.insert(key, value);
///
///         // Write back to the file
///         let content = serde_json::to_string_pretty(&map)
///             .map_err(|e| PersistentError::Serde(e))?;
///
///         fs::write(&self.path, content)?;
///
///         Ok(())
///     }
///
///     async fn delete(&self, key: &K) -> Result<(), PersistentError> {
///         // Ensure the file exists
///         self.ensure_file_exists()?;
///
///         // Load existing data
///         let mut map: HashMap<K, V> = self.load_all().await?;
///
///         // Remove the key
///         map.remove(key);
///
///         // Write back to the file
///         let content = serde_json::to_string_pretty(&map)
///             .map_err(|e| PersistentError::Serde(e))?;
///
///         fs::write(&self.path, content)?;
///
///         Ok(())
///     }
///
///     async fn flush(&self) -> Result<(), PersistentError> {
///         // No buffering in this implementation, so nothing to flush
///         Ok(())
///     }
/// }
/// ```
///
/// # Best Practices for Custom Backends
///
/// 1. **Error Handling**: Convert backend-specific errors to `PersistentError`
/// 2. **Concurrency**: Ensure your backend is safe for concurrent access
/// 3. **Performance**: Consider caching or batching operations for better performance
/// 4. **Resilience**: Handle edge cases like missing files or corrupted data gracefully
/// 5. **Testing**: Create tests that verify persistence across application restarts
#[async_trait::async_trait]
pub trait StorageBackend<K, V>
where
    K: Eq + Hash + Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Load all key-value pairs from the storage backend.
    ///
    /// This method is called when initializing a `PersistentMap` to populate
    /// the in-memory map with existing data.
    ///
    /// # Errors
    ///
    /// Returns a `PersistentError` if loading fails for any reason, such as:
    /// - The storage location doesn't exist
    /// - The data is corrupted or in an invalid format
    /// - There are permission issues
    ///
    /// # Implementation Notes
    ///
    /// - This method should be idempotent and safe to call multiple times
    /// - If the storage is empty or doesn't exist yet, return an empty `HashMap`
    /// - Consider adding error recovery mechanisms for corrupted data
    async fn load_all(&self) -> Result<HashMap<K, V>, PersistentError>;

    /// Save a key-value pair to the storage backend.
    ///
    /// This method is called whenever a key-value pair is inserted into the map.
    ///
    /// # Errors
    ///
    /// Returns a `PersistentError` if saving fails for any reason, such as:
    /// - The storage location is not writable
    /// - There are permission issues
    /// - The backend has reached capacity
    ///
    /// # Implementation Notes
    ///
    /// - This method should be atomic if possible
    /// - Consider batching or caching writes for better performance
    /// - If your backend requires serialization, handle serialization errors appropriately
    async fn save(&self, key: K, value: V) -> Result<(), PersistentError>;

    /// Delete a key-value pair from the storage backend.
    ///
    /// This method is called whenever a key-value pair is removed from the map.
    ///
    /// # Errors
    ///
    /// Returns a `PersistentError` if deletion fails for any reason, such as:
    /// - The storage location is not writable
    /// - There are permission issues
    ///
    /// # Implementation Notes
    ///
    /// - This method should be idempotent - deleting a non-existent key should not be an error
    /// - Consider optimizing for the case where the key doesn't exist
    async fn delete(&self, key: &K) -> Result<(), PersistentError>;

    /// Flush any buffered writes to the storage backend.
    ///
    /// This method is called when the user explicitly requests to ensure all data is persisted.
    ///
    /// # Errors
    ///
    /// Returns a `PersistentError` if flushing fails for any reason, such as:
    /// - The storage location is not writable
    /// - There are permission issues
    ///
    /// # Implementation Notes
    ///
    /// - This method is optional and has a default implementation that does nothing
    /// - Backends that buffer writes should override this method to ensure data is persisted
    /// - This method should be idempotent and safe to call multiple times
    async fn flush(&self) -> Result<(), PersistentError> {
        Ok(())
    }

    /// Check if a key exists in the storage backend.
    ///
    /// This is an optional method with a default implementation that loads all data
    /// and checks if the key exists. Backend implementations can override this
    /// for better performance if they can check for existence without loading all data.
    ///
    /// # Errors
    ///
    /// Returns a `PersistentError` if the check fails for any reason.
    ///
    /// # Implementation Notes
    ///
    /// - The default implementation is inefficient for large datasets
    /// - Override this method if your backend can check for existence more efficiently
    async fn contains_key(&self, key: &K) -> Result<bool, PersistentError> {
        let all = self.load_all().await?;
        Ok(all.contains_key(key))
    }

    /// Get the number of key-value pairs in the storage backend.
    ///
    /// This is an optional method with a default implementation that loads all data
    /// and counts the entries. Backend implementations can override this
    /// for better performance if they can count entries without loading all data.
    ///
    /// # Errors
    ///
    /// Returns a `PersistentError` if the count operation fails for any reason.
    ///
    /// # Implementation Notes
    ///
    /// - The default implementation is inefficient for large datasets
    /// - Override this method if your backend can count entries more efficiently
    async fn len(&self) -> Result<usize, PersistentError> {
        let all = self.load_all().await?;
        Ok(all.len())
    }

    /// Check if the storage backend is empty.
    ///
    /// This is an optional method with a default implementation that uses `len()`.
    /// Backend implementations can override this for better performance.
    ///
    /// # Errors
    ///
    /// Returns a `PersistentError` if the check fails for any reason.
    ///
    /// # Implementation Notes
    ///
    /// - The default implementation uses `len()`, which may be inefficient
    /// - Override this method if your backend can check emptiness more efficiently
    async fn is_empty(&self) -> Result<bool, PersistentError> {
        Ok(self.len().await? == 0)
    }
}

/// Errors that can occur when using `PersistentMap`.
///
/// This enum represents all the possible errors that can occur when using
/// the various storage backends.
#[derive(Error, Debug)]
pub enum PersistentError {
    /// An error occurred in the `SQLite` backend.
    #[cfg(feature = "sqlite")]
    #[error("sqlite error: {0}")]
    Sqlite(#[from] tokio_rusqlite::Error),

    /// An error occurred in the CSV backend.
    #[cfg(feature = "csv_backend")]
    #[error("csv error: {0}")]
    Csv(String),

    /// An I/O error occurred.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A serialization or deserialization error occurred.
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    /// An error occurred in the Sled backend.
    #[cfg(feature = "sled_backend")]
    #[error("sled error: {0}")]
    Sled(#[from] sled::Error),
}

/// Shorthand Result with error defaulting to `PersistentError`.
pub type Result<T, E = PersistentError> = std::result::Result<T, E>;

// Re-export backends
#[cfg(feature = "csv_backend")]
pub use crate::backends::csv;

#[cfg(feature = "in_memory")]
pub use crate::backends::in_memory;

#[cfg(feature = "sqlite")]
pub use crate::backends::sqlite;

mod backends;

/// A persistent key-value map with in-memory caching.
///
/// `PersistentMap` combines a fast in-memory `DashMap` with a persistent
/// storage backend. It provides a simple API for storing and retrieving
/// key-value pairs, with automatic persistence.
///
/// # Type Parameters
///
/// * `K`: The key type, which must be hashable, serializable, and cloneable
/// * `V`: The value type, which must be serializable and cloneable
/// * `B`: The storage backend type, which must implement `StorageBackend<K, V>`
///
/// # Examples
///
/// ```rust,no_run
/// use persistent_map::{PersistentMap, Result};
/// # #[cfg(feature = "sqlite")]
/// use persistent_map::sqlite::SqliteBackend;
///
/// # #[cfg(feature = "sqlite")]
/// # async fn example() -> Result<()> {
/// # // Create a SQLite backend
/// # let backend = SqliteBackend::new("my_database.db").await?;
/// #
/// # // Initialize the persistent map
/// # let map = PersistentMap::new(backend).await?;
/// #
/// # // Insert a key-value pair (persists automatically)
/// # map.insert("hello".to_string(), "world".to_string()).await?;
/// #
/// # // Retrieve a value (from memory)
/// # assert_eq!(map.get(&"hello".to_string()), Some("world".to_string()));
/// #
/// # // Remove a key-value pair (removes from persistence too)
/// # let old_value = map.remove(&"hello".to_string()).await?;
/// # assert_eq!(old_value, Some("world".to_string()));
/// # Ok(())
/// # }
/// #
/// # #[cfg(not(feature = "sqlite"))]
/// # fn example() {}
/// ```
pub struct PersistentMap<K, V, B>
where
    K: Eq + Hash + Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
    B: StorageBackend<K, V> + Send + Sync + 'static,
{
    /// The in-memory map for fast access
    map: DashMap<K, V>,

    /// The storage backend for persistence
    backend: B,
}

impl<K, V, B> PersistentMap<K, V, B>
where
    K: Eq + Hash + Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
    B: StorageBackend<K, V> + Send + Sync + 'static,
{
    /// Creates a new `PersistentMap` with the given storage backend.
    ///
    /// This method initializes the map and loads all existing key-value pairs
    /// from the storage backend into memory.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use persistent_map::{PersistentMap, Result};
    /// # #[cfg(feature = "sqlite")]
    /// use persistent_map::sqlite::SqliteBackend;
    ///
    /// # #[cfg(feature = "sqlite")]
    /// # async fn example() -> Result<()> {
    /// # let backend = SqliteBackend::new("my_database.db").await?;
    /// # let map: PersistentMap<String, String, _> = PersistentMap::new(backend).await?;
    /// # Ok(())
    /// # }
    /// #
    /// # #[cfg(not(feature = "sqlite"))]
    /// # fn example() {}
    /// ```
    /// # Errors
    ///
    /// Returns an error if loading from the backend fails.
    #[inline]
    pub async fn new(backend: B) -> Result<Self> {
        let map = DashMap::new();
        let pm = Self { map, backend };
        pm.load().await?;
        Ok(pm)
    }

    /// Loads all key-value pairs from the storage backend into memory.
    ///
    /// This method is called automatically when creating a new `PersistentMap`,
    /// but can also be called manually to refresh the in-memory cache.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use persistent_map::{PersistentMap, StorageBackend, Result};
    /// #
    /// # async fn example(map: PersistentMap<String, String, impl StorageBackend<String, String> + Send + Sync>) -> Result<()> {
    /// // Reload all data from the storage backend
    /// map.load().await?;
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if loading from the backend fails.
    #[inline]
    pub async fn load(&self) -> Result<(), PersistentError> {
        let all = self.backend.load_all().await?;
        for (k, v) in all {
            self.map.insert(k, v);
        }
        Ok(())
    }

    /// Inserts a key-value pair into the map and persists it to the storage backend.
    ///
    /// If the map already contains the key, the value is updated and the old value
    /// is returned. Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use persistent_map::{PersistentMap, StorageBackend, Result};
    /// #
    /// # async fn example(map: PersistentMap<String, String, impl StorageBackend<String, String> + Send + Sync>) -> Result<()> {
    /// // Insert a new key-value pair
    /// let old = map.insert("key".to_string(), "value".to_string()).await?;
    /// assert_eq!(old, None);
    ///
    /// // Update an existing key
    /// let old = map.insert("key".to_string(), "new value".to_string()).await?;
    /// assert_eq!(old, Some("value".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if saving to the backend fails.
    #[inline]
    pub async fn insert(&self, key: K, value: V) -> Result<Option<V>> {
        let old = self.map.insert(key.clone(), value.clone());
        self.backend.save(key, value).await?;
        Ok(old)
    }

    /// Retrieves a value from the map by its key.
    ///
    /// This method only accesses the in-memory map and does not interact with
    /// the storage backend, making it very fast.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use persistent_map::{PersistentMap, StorageBackend};
    /// #
    /// # fn example(map: PersistentMap<String, String, impl StorageBackend<String, String> + Send + Sync>) {
    /// // Get a value
    /// if let Some(value) = map.get(&"key".to_string()) {
    ///     println!("Value: {}", value);
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn get(&self, key: &K) -> Option<V> {
        self.map.get(key).map(|r| r.value().clone())
    }

    /// Removes a key-value pair from the map and the storage backend.
    ///
    /// If the map contains the key, the key-value pair is removed and the old value
    /// is returned. Otherwise, `None` is returned.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use persistent_map::{PersistentMap, StorageBackend, Result};
    /// #
    /// # async fn example(map: PersistentMap<String, String, impl StorageBackend<String, String> + Send + Sync>) -> Result<()> {
    /// // Remove a key-value pair
    /// let old = map.remove(&"key".to_string()).await?;
    /// if let Some(value) = old {
    ///     println!("Removed value: {}", value);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if deleting from the backend fails.
    #[inline]
    pub async fn remove(&self, key: &K) -> Result<Option<V>> {
        let old = self.map.remove(key).map(|(_, v)| v);
        if old.is_some() {
            match self.backend.delete(key).await {
                Ok(()) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(old)
    }

    /// Returns the number of key-value pairs in the map.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use persistent_map::{PersistentMap, StorageBackend};
    /// #
    /// # fn example(map: PersistentMap<String, String, impl StorageBackend<String, String> + Send + Sync>) {
    /// let count = map.len();
    /// println!("Map contains {} entries", count);
    /// # }
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map contains no key-value pairs.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use persistent_map::{PersistentMap, StorageBackend};
    /// #
    /// # fn example(map: PersistentMap<String, String, impl StorageBackend<String, String> + Send + Sync>) {
    /// if map.is_empty() {
    ///     println!("Map is empty");
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns `true` if the map contains the specified key.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use persistent_map::{PersistentMap, StorageBackend};
    /// #
    /// # fn example(map: PersistentMap<String, String, impl StorageBackend<String, String> + Send + Sync>) {
    /// if map.contains_key(&"key".to_string()) {
    ///     println!("Map contains the key");
    /// }
    /// # }
    /// ```
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Clears the in-memory map without affecting the storage backend.
    ///
    /// This method only clears the in-memory cache and does not delete any data
    /// from the storage backend. To completely clear the storage, you should
    /// delete the underlying storage file or database.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use persistent_map::{PersistentMap, StorageBackend};
    /// #
    /// # fn example(map: PersistentMap<String, String, impl StorageBackend<String, String> + Send + Sync>) {
    /// // Clear the in-memory cache
    /// map.clear();
    /// assert_eq!(map.len(), 0);
    /// # }
    /// ```
    #[inline]
    pub fn clear(&self) {
        self.map.clear();
    }

    /// Flushes any buffered writes to the storage backend.
    ///
    /// This method is useful for backends that buffer writes for performance.
    /// It ensures that all data is persisted to the storage medium.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use persistent_map::{PersistentMap, StorageBackend, Result};
    /// #
    /// # async fn example(map: PersistentMap<String, String, impl StorageBackend<String, String> + Send + Sync>) -> Result<()> {
    /// // Ensure all data is persisted
    /// map.flush().await?;
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if flushing the backend fails.
    #[inline]
    pub async fn flush(&self) -> Result<(), PersistentError> {
        self.backend.flush().await
    }

    /// Returns a reference to the storage backend.
    ///
    /// This method is useful for accessing backend-specific functionality.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use persistent_map::{PersistentMap, StorageBackend};
    /// #
    /// # fn example<B>(map: PersistentMap<String, String, B>)
    /// # where B: StorageBackend<String, String> + Send + Sync
    /// # {
    /// let backend = map.backend();
    /// // Use backend-specific functionality
    /// # }
    /// ```
    #[inline]
    pub const fn backend(&self) -> &B {
        &self.backend
    }
}
