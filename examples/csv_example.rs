#[cfg(feature = "csv_backend")]
use persistent_map::{PersistentMap, Result};

#[cfg(feature = "csv_backend")]
use persistent_map::csv::CsvBackend;

#[cfg(feature = "csv_backend")]
#[tokio::main]
async fn main() -> Result<()> {
    // Create a CSV backend (file will be created automatically)
    let backend = CsvBackend::new("example_data.csv");

    // Initialize the map with the backend
    let map = PersistentMap::new(backend).await?;

    println!("Map initialized with {} entries", map.len());

    // Insert some data
    map.insert("greeting".to_string(), "Hello, CSV World!".to_string())
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
    println!("Run this example again to see that data persists between runs!");

    Ok(())
}

#[cfg(not(feature = "csv_backend"))]
fn main() {
    println!("This example requires the 'csv_backend' feature to be enabled.");
    println!("Add the following to your Cargo.toml:");
    println!("persistent-map = {{ version = \"0.1.0\", features = [\"csv_backend\"] }}");
}
