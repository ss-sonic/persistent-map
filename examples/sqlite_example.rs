use persistent_map::sqlite::SqliteBackend;
use persistent_map::{PersistentMap, Result};

#[cfg(feature = "sqlite")]
#[tokio::main]
async fn main() -> Result<()> {
    // Create a SQLite backend
    let backend = SqliteBackend::new("example_sqlite.db").await?;

    // Initialize the map with the backend
    let map = PersistentMap::new(backend).await?;

    println!("Map initialized with {} entries", map.len());

    // Insert some data (persists automatically)
    map.insert(
        "greeting".to_string(),
        "Hello, Persistent World!".to_string(),
    )
    .await?;
    map.insert("count".to_string(), "42".to_string()).await?;
    map.insert("pi".to_string(), "3.14159".to_string()).await?;

    println!("Inserted 3 entries");

    // Retrieve data (from in-memory cache)
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
    println!("Run this example again to see that data persists between runs!");

    Ok(())
}

#[cfg(not(feature = "sqlite"))]
fn main() {
    println!("This example requires the 'sqlite' feature to be enabled.");
    println!("Add the following to your Cargo.toml:");
    println!("persistent-map = {{ version = \"0.1.0\", features = [\"sqlite\"] }}");
}
