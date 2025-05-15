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

## Custom Backends

You can implement your own storage backend by implementing the `StorageBackend` trait:

```rust
use persistent_map::{StorageBackend, PersistentError, Result};
use std::collections::HashMap;
use serde::{Serialize, de::DeserializeOwned};
use std::hash::Hash;

struct MyCustomBackend {
    // Your backend-specific fields
}

#[async_trait::async_trait]
impl<K, V> StorageBackend<K, V> for MyCustomBackend
where
    K: Eq + Hash + Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    async fn load_all(&self) -> Result<HashMap<K, V>, PersistentError> {
        // Implementation for loading all key-value pairs
        Ok(HashMap::new())
    }

    async fn save(&self, key: K, value: V) -> Result<(), PersistentError> {
        // Implementation for saving a key-value pair
        Ok(())
    }

    async fn delete(&self, key: &K) -> Result<(), PersistentError> {
        // Implementation for deleting a key-value pair
        Ok(())
    }

    async fn flush(&self) -> Result<(), PersistentError> {
        // Implementation for flushing buffered writes
        Ok(())
    }
}
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
