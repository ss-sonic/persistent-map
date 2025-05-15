# Persistent-Map: Effortless Persistent, Async Key-Value Store for Rust

Persistent-Map offers a simple and efficient way to store and retrieve key-value data that needs to survive application restarts. It provides an easy-to-use API with an asynchronous interface and pluggable storage backends.

## Key Features

- **Simple API:** Familiar `insert` and `get` operations with an async interface.
- **Multiple Backends:** SQLite, CSV, in-memory, and extensible for more.
- **Asynchronous:** Designed with `async/await` for non-blocking operations.
- **In-Memory Cache:** Uses `DashMap` for fast concurrent access to frequently used data.
- **Generic:** Works with types that implement `Serialize` and `DeserializeOwned`.
- **Flexible:** Choose the storage backend that best fits your needs.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
persistent-map = "0.1.0" # Replace with the latest version
tokio = { version = "1", features = ["macros", "rt-multi-thread"] } # For the async runtime

# Choose the backends you need
persistent-map = { version = "0.1.0", features = ["sqlite", "csv_backend", "in_memory"] }
```

By default, the crate includes the SQLite and in-memory backends. You can enable other backends as needed.

## Basic Usage

Here's a simple example using the SQLite backend:

```rust
use persistent_map::{PersistentMap, sqlite::SqliteBackend, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a SQLite backend
    let backend = SqliteBackend::new("my_app_data.db").await?;

    // Initialize the map with the backend
    let map = PersistentMap::new(backend).await?;

    // Insert some data (persists automatically)
    map.insert("greeting".to_string(), "Hello, Persistent World!".to_string()).await?;
    map.insert("count".to_string(), "42".to_string()).await?;

    // Retrieve data (from in-memory cache)
    if let Some(value) = map.get(&"greeting".to_string()) {
        println!("Greeting: {}", value);
    }

    // Check if a key exists
    if map.contains_key(&"count".to_string()) {
        println!("Count exists!");
    }

    // Remove a key-value pair
    let old_value = map.remove(&"greeting".to_string()).await?;
    println!("Removed value: {:?}", old_value);

    // Ensure all data is persisted
    map.flush().await?;

    println!("Data persisted. Map contains {} entries.", map.len());

    Ok(())
}
```

## Available Backends

### SQLite Backend

The SQLite backend is ideal for most applications, providing reliable persistence with good performance.

```rust
use persistent_map::{PersistentMap, sqlite::SqliteBackend, Result};

async fn example() -> Result<()> {
    let backend = SqliteBackend::new("my_database.db").await?;
    let map = PersistentMap::new(backend).await?;
    // Use the map...
    Ok(())
}
```

### CSV Backend

The CSV backend stores data in a simple CSV file, which can be useful for data that needs to be human-readable.

```rust
use persistent_map::{PersistentMap, csv::CsvBackend, Result};

async fn example() -> Result<()> {
    let backend = CsvBackend::new("my_data.csv");
    let map = PersistentMap::new(backend).await?;
    // Use the map...
    Ok(())
}
```

### In-Memory Backend

The in-memory backend doesn't provide persistence but can be useful for testing or temporary storage.

```rust
use persistent_map::{PersistentMap, in_memory::InMemoryBackend, Result};

async fn example() -> Result<()> {
    let backend = InMemoryBackend::new();
    let map = PersistentMap::new(backend).await?;
    // Use the map...
    Ok(())
}
```

## Implementing Custom Backends

One of the key features of persistent-map is its extensibility. You can create your own storage backends by implementing the `StorageBackend` trait.

### The StorageBackend Trait

The `StorageBackend` trait defines the interface that all storage backends must implement:

```rust
#[async_trait::async_trait]
pub trait StorageBackend<K, V>
where
    K: Eq + Hash + Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    // Required methods
    async fn load_all(&self) -> Result<HashMap<K, V>, PersistentError>;
    async fn save(&self, key: K, value: V) -> Result<(), PersistentError>;
    async fn delete(&self, key: &K) -> Result<(), PersistentError>;

    // Optional methods with default implementations
    async fn flush(&self) -> Result<(), PersistentError> { Ok(()) }
    async fn contains_key(&self, key: &K) -> Result<bool, PersistentError>;
    async fn len(&self) -> Result<usize, PersistentError>;
    async fn is_empty(&self) -> Result<bool, PersistentError>;
}
```

### Example: JSON File Backend

Here's an example of a custom backend that stores data in a JSON file:

```rust
use persistent_map::{StorageBackend, PersistentError, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use serde::{Serialize, de::DeserializeOwned};
use std::hash::Hash;

struct JsonFileBackend {
    path: PathBuf,
}

impl JsonFileBackend {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    // Helper method to ensure the file exists
    fn ensure_file_exists(&self) -> std::io::Result<()> {
        if !self.path.exists() {
            // Create parent directories if they don't exist
            if let Some(parent) = self.path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            // Create the file with an empty JSON object
            fs::write(&self.path, "{}")?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<K, V> StorageBackend<K, V> for JsonFileBackend
where
    K: Eq + Hash + Clone + Serialize + DeserializeOwned + Send + Sync + ToString + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    async fn load_all(&self) -> Result<HashMap<K, V>, PersistentError> {
        // Ensure the file exists
        self.ensure_file_exists()?;

        // If the file is empty or contains just "{}", return an empty HashMap
        let content = fs::read_to_string(&self.path)?;
        if content.trim() == "{}" {
            return Ok(HashMap::new());
        }

        // Parse the JSON file
        let map = serde_json::from_str(&content)
            .map_err(|e| PersistentError::Serde(e))?;

        Ok(map)
    }

    async fn save(&self, key: K, value: V) -> Result<(), PersistentError> {
        // Ensure the file exists
        self.ensure_file_exists()?;

        // Load existing data
        let mut map = self.load_all().await?;

        // Update the map
        map.insert(key, value);

        // Write back to the file
        let content = serde_json::to_string_pretty(&map)
            .map_err(|e| PersistentError::Serde(e))?;

        fs::write(&self.path, content)?;

        Ok(())
    }

    async fn delete(&self, key: &K) -> Result<(), PersistentError> {
        // Ensure the file exists
        self.ensure_file_exists()?;

        // Load existing data
        let mut map = self.load_all().await?;

        // Remove the key
        map.remove(key);

        // Write back to the file
        let content = serde_json::to_string_pretty(&map)
            .map_err(|e| PersistentError::Serde(e))?;

        fs::write(&self.path, content)?;

        Ok(())
    }
}
```

### Best Practices for Custom Backends

When implementing a custom backend, consider the following best practices:

1. **Error Handling**: Convert backend-specific errors to `PersistentError`
2. **Concurrency**: Ensure your backend is safe for concurrent access
3. **Performance**: Consider caching or batching operations for better performance
4. **Resilience**: Handle edge cases like missing files or corrupted data gracefully
5. **Testing**: Create tests that verify persistence across application restarts

### Publishing Your Custom Backend

If you've created a useful backend implementation, consider publishing it as a separate crate that depends on persistent-map. This allows others to benefit from your work while keeping the core crate lightweight.

For example:

```
persistent-map-redis = { version = "0.1.0", dependencies = { persistent-map = "0.1.0", redis = "0.21.0" } }
```

## Performance Considerations

- The in-memory `DashMap` provides fast concurrent access to data
- Persistence operations are asynchronous and don't block the main thread
- For best performance with frequent writes, consider calling `flush()` periodically rather than after every write

## Future Enhancements

- Additional storage backends (Postgres, Redis, etc.)
- Transactional operations
- Batch operations for improved performance
- Iterator support for more idiomatic Rust usage

## Contributing

Contributions are welcome! Here are some ways you can contribute:

- Implement new storage backends
- Improve documentation
- Add tests
- Report bugs
- Suggest new features

Please open an issue or submit a pull request on [GitHub](https://github.com/routerprotocol/persistent-map).

## Versioning

This project follows [Semantic Versioning](https://semver.org/). The current version is 0.1.0, which means it is still in initial development and the API may change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
