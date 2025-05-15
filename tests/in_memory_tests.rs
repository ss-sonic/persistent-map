#[cfg(feature = "in_memory")]
mod tests {
    use persistent_map::{PersistentMap, Result};

    #[tokio::test]
    async fn test_in_memory_backend() -> Result<()> {
        // Create an in-memory backend
        let backend = persistent_map::in_memory::InMemoryBackend::new();

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

        // Test flush (should be a no-op for in-memory)
        map.flush().await?;
        assert_eq!(map.len(), 3);

        // Test clear
        map.clear();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());

        Ok(())
    }
}
