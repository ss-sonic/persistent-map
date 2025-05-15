//! SQLite backend implementation for PersistentMap.
//!
//! This module provides a SQLite-based storage backend for PersistentMap.
//! It uses tokio-rusqlite for asynchronous SQLite operations.

use crate::StorageBackend;
use crate::{PersistentError, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, hash::Hash, str::FromStr};
use tokio_rusqlite::{params, Connection};

/// A SQLite-based storage backend for PersistentMap.
///
/// This backend stores key-value pairs in a SQLite database, providing
/// durable persistence with good performance characteristics.
///
/// # Examples
///
/// ```rust,no_run
/// use persistent_map::{PersistentMap, Result};
/// use persistent_map::sqlite::SqliteBackend;
///
/// # async fn example() -> Result<()> {
/// // Create a SQLite backend
/// let backend = SqliteBackend::new("my_database.db").await?;
///
/// // Initialize a PersistentMap with the backend
/// let map: PersistentMap<String, String, _> = PersistentMap::new(backend).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SqliteBackend {
    /// The SQLite connection
    conn: Connection,
}

impl SqliteBackend {
    /// Creates a new SQLite backend with the given database path.
    ///
    /// This method opens a connection to the SQLite database at the specified path
    /// and creates the necessary table if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `db_path` - The path to the SQLite database file
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `SqliteBackend` or an error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use persistent_map::sqlite::SqliteBackend;
    /// use persistent_map::Result;
    ///
    /// # async fn example() -> Result<()> {
    /// let backend = SqliteBackend::new("my_database.db").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path).await?;
        conn.call(|c| {
            c.execute(
                "CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT NOT NULL)",
                [],
            )
            .map_err(tokio_rusqlite::Error::Rusqlite)
        })
        .await?;

        // Create an index for faster lookups if it doesn't exist
        conn.call(|c| {
            c.execute("CREATE INDEX IF NOT EXISTS kv_key_idx ON kv (key)", [])
                .map_err(tokio_rusqlite::Error::Rusqlite)
        })
        .await?;

        Ok(Self { conn })
    }

    /// Returns the path to the SQLite database file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the path to the database file or an error
    pub async fn db_path(&self) -> Result<String> {
        let result = self
            .conn
            .call(|c| {
                c.query_row("PRAGMA database_list", [], |row| {
                    let path: String = row.get(2)?;
                    Ok(path)
                })
                .map_err(tokio_rusqlite::Error::Rusqlite)
            })
            .await?; // Use ? to convert the error type

        Ok(result)
    }
}

/// Implementation of the `StorageBackend` trait for `SqliteBackend`.
///
/// This implementation provides methods for loading, saving, and deleting
/// key-value pairs from a SQLite database.
#[async_trait::async_trait]
impl<K, V> StorageBackend<K, V> for SqliteBackend
where
    K: Eq
        + Hash
        + Clone
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + 'static
        + ToString
        + FromStr,
    <K as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Loads all key-value pairs from the SQLite database.
    ///
    /// This method queries the database for all key-value pairs and deserializes
    /// them into the appropriate types.
    async fn load_all(&self) -> Result<HashMap<K, V>, PersistentError> {
        let rows = self
            .conn
            .call(|c| {
                let mut stmt = c.prepare_cached("SELECT key, value FROM kv")?;
                let mut map = HashMap::with_capacity(100); // Pre-allocate for better performance
                let mut rows_iter = stmt.query_map([], |r| {
                    let key_str: String = r.get(0)?;
                    let val_str: String = r.get(1)?;
                    Ok((key_str, val_str))
                })?;

                while let Some(Ok((k_str, v_str))) = rows_iter.next() {
                    // Deserialize the value from JSON
                    let value: V = serde_json::from_str(&v_str)
                        .map_err(|e| tokio_rusqlite::Error::Other(Box::new(e)))?;

                    // Parse the key from string
                    let key = k_str
                        .parse()
                        .map_err(|e| tokio_rusqlite::Error::Other(Box::new(e)))?;

                    map.insert(key, value);
                }
                Ok(map)
            })
            .await?;
        Ok(rows)
    }

    /// Saves a key-value pair to the SQLite database.
    ///
    /// This method serializes the key and value to strings and inserts or
    /// replaces them in the database.
    async fn save(&self, key: K, value: V) -> Result<(), PersistentError> {
        let key_str = key.to_string();
        let val_json = serde_json::to_string(&value)?;

        self.conn
            .call(move |c| {
                c.execute(
                    "INSERT OR REPLACE INTO kv (key, value) VALUES (?1, ?2)",
                    params![key_str, val_json],
                )
                .map_err(tokio_rusqlite::Error::Rusqlite)
            })
            .await?;

        Ok(())
    }

    /// Deletes a key-value pair from the SQLite database.
    ///
    /// This method removes the key-value pair with the specified key from the database.
    async fn delete(&self, key: &K) -> Result<(), PersistentError> {
        let key_str = key.to_string();

        self.conn
            .call(move |c| {
                c.execute("DELETE FROM kv WHERE key = ?1", params![key_str])
                    .map_err(tokio_rusqlite::Error::Rusqlite)
            })
            .await?;

        Ok(())
    }

    /// Flushes any buffered writes to the SQLite database.
    ///
    /// This method ensures that all data is written to disk by executing
    /// a PRAGMA synchronous command.
    async fn flush(&self) -> Result<(), PersistentError> {
        self.conn
            .call(|c| {
                c.execute("PRAGMA synchronous = FULL", [])
                    .map_err(tokio_rusqlite::Error::Rusqlite)
            })
            .await?;

        Ok(())
    }
}
