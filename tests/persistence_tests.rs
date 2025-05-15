#[cfg(feature = "sqlite")]
mod sqlite_persistence {
    use persistent_map::{PersistentMap, Result};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_sqlite_persistence() -> Result<()> {
        // Create a temporary directory for the test
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_persistence.db");
        let db_path_str = db_path.to_str().unwrap();

        // First session: Create a map and insert data
        {
            let backend = persistent_map::sqlite::SqliteBackend::new(db_path_str).await?;
            let map: PersistentMap<String, String, _> = PersistentMap::new(backend).await?;

            // Insert some data
            map.insert("key1".to_string(), "value1".to_string()).await?;
            map.insert("key2".to_string(), "value2".to_string()).await?;

            // Ensure data is persisted
            map.flush().await?;

            // Map is dropped here, closing the connection
        }

        // Second session: Create a new map and verify data is still there
        {
            let backend = persistent_map::sqlite::SqliteBackend::new(db_path_str).await?;
            let map: PersistentMap<String, String, _> = PersistentMap::new(backend).await?;

            // Verify data was persisted
            assert_eq!(map.get(&"key1".to_string()), Some("value1".to_string()));
            assert_eq!(map.get(&"key2".to_string()), Some("value2".to_string()));
            assert_eq!(map.len(), 2);
        }

        // Clean up
        dir.close().unwrap();

        Ok(())
    }
}

#[cfg(feature = "csv_backend")]
mod csv_persistence {
    use persistent_map::{PersistentMap, Result};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_csv_persistence() -> Result<()> {
        // Create a temporary directory for the test
        let dir = tempdir().unwrap();
        let csv_path = dir.path().join("test_persistence.csv");
        let csv_path_str = csv_path.to_str().unwrap();

        // First session: Create a map and insert data
        {
            let backend = persistent_map::csv::CsvBackend::new(csv_path_str);
            let map: PersistentMap<String, String, _> = PersistentMap::new(backend).await?;

            // Insert some data
            map.insert("key1".to_string(), "value1".to_string()).await?;
            map.insert("key2".to_string(), "value2".to_string()).await?;

            // Ensure data is persisted
            map.flush().await?;

            // Map is dropped here
        }

        // Second session: Create a new map and verify data is still there
        {
            let backend = persistent_map::csv::CsvBackend::new(csv_path_str);
            let map: PersistentMap<String, String, _> = PersistentMap::new(backend).await?;

            // Verify data was persisted
            assert_eq!(map.get(&"key1".to_string()), Some("value1".to_string()));
            assert_eq!(map.get(&"key2".to_string()), Some("value2".to_string()));
            assert_eq!(map.len(), 2);
        }

        // Clean up
        dir.close().unwrap();

        Ok(())
    }
}

#[cfg(feature = "in_memory")]
mod in_memory_persistence {
    use persistent_map::{PersistentMap, Result};

    #[tokio::test]
    async fn test_in_memory_no_persistence() -> Result<()> {
        // First session: Create a map and insert data
        {
            let backend = persistent_map::in_memory::InMemoryBackend::new();
            let map: PersistentMap<String, String, _> = PersistentMap::new(backend).await?;

            // Insert some data
            map.insert("key1".to_string(), "value1".to_string()).await?;
            map.insert("key2".to_string(), "value2".to_string()).await?;

            // Ensure data is "persisted" (no-op for in-memory)
            map.flush().await?;

            // Map is dropped here
        }

        // Second session: Create a new map and verify data is NOT there (in-memory doesn't persist)
        {
            let backend = persistent_map::in_memory::InMemoryBackend::new();
            let map: PersistentMap<String, String, _> = PersistentMap::new(backend).await?;

            // Verify data was NOT persisted (in-memory doesn't persist)
            assert_eq!(map.get(&"key1".to_string()), None);
            assert_eq!(map.get(&"key2".to_string()), None);
            assert_eq!(map.len(), 0);
        }

        Ok(())
    }
}
