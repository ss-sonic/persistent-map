use persistent_map::{PersistentMap, Result};
use std::collections::HashMap;

// Create a simple in-memory backend
struct InMemoryBackend {
    data: HashMap<String, String>,
}

impl InMemoryBackend {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl persistent_map::StorageBackend<String, String> for InMemoryBackend {
    async fn load_all(&self) -> Result<HashMap<String, String>> {
        Ok(self.data.clone())
    }

    async fn save(&self, key: String, value: String) -> Result<()> {
        // In a real implementation, this would actually save the data
        println!("Saving: {} = {}", key, value);
        Ok(())
    }

    async fn delete(&self, key: &String) -> Result<()> {
        // In a real implementation, this would actually delete the data
        println!("Deleting: {}", key);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create an in-memory backend
    let backend = InMemoryBackend::new();

    // Initialize the map with the backend
    let map = PersistentMap::new(backend).await?;

    println!("Map initialized with {} entries", map.len());

    // Insert some data
    map.insert(
        "greeting".to_string(),
        "Hello, Persistent World!".to_string(),
    )
    .await?;
    map.insert("count".to_string(), "42".to_string()).await?;
    map.insert("pi".to_string(), "3.14159".to_string()).await?;

    println!("Inserted 3 entries");

    // Retrieve data
    if let Some(value) = map.get(&"greeting".to_string()) {
        println!("Greeting: {}", value);
    }

    // Check if a key exists
    if map.contains_key(&"count".to_string()) {
        println!("Count exists!");
        if let Some(count) = map.get(&"count".to_string()) {
            println!("Count value: {}", count);
        }
    }

    // Remove a key-value pair
    let old_value = map.remove(&"pi".to_string()).await?;
    println!("Removed pi value: {:?}", old_value);

    // Ensure all data is persisted
    map.flush().await?;

    println!("Data persisted. Map contains {} entries.", map.len());

    Ok(())
}
