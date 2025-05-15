#[cfg(feature = "csv_backend")]
mod tests {
    use persistent_map::{PersistentMap, Result};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_csv_backend() -> Result<()> {
        // Create a temporary directory for the test
        let dir = tempdir().unwrap();
        let csv_path = dir.path().join("test.csv");
        let csv_path_str = csv_path.to_str().unwrap();

        // Create a CSV backend (file will be created automatically)
        let backend = persistent_map::csv::CsvBackend::new(csv_path_str);

        // Initialize the map with the backend
        let map = PersistentMap::new(backend).await?;

        // Test initial state
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());

        // Test insert
        map.insert("key1".to_string(), "value1".to_string()).await?;
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
        assert!(map.contains_key(&"key1".to_string()));
        assert_eq!(map.get(&"key1".to_string()), Some("value1".to_string()));

        // Test update
        map.insert("key1".to_string(), "value2".to_string()).await?;
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&"key1".to_string()), Some("value2".to_string()));

        // Test remove
        let old_value = map.remove(&"key1".to_string()).await?;
        assert_eq!(old_value, Some("value2".to_string()));
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert!(!map.contains_key(&"key1".to_string()));

        // Test multiple inserts
        map.insert("key1".to_string(), "value1".to_string()).await?;
        map.insert("key2".to_string(), "value2".to_string()).await?;
        map.insert("key3".to_string(), "value3".to_string()).await?;
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(map.get(&"key2".to_string()), Some("value2".to_string()));
        assert_eq!(map.get(&"key3".to_string()), Some("value3".to_string()));

        // Test flush
        map.flush().await?;

        // Clean up
        drop(map);
        dir.close().unwrap();

        Ok(())
    }
}
