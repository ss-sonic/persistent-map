use crate::{PersistentError, Result, StorageBackend};
use csv::{ReaderBuilder, WriterBuilder};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, fs::OpenOptions, hash::Hash, path::PathBuf};

pub struct CsvBackend {
    path: PathBuf,
}

impl CsvBackend {
    /// Creates a new CSV backend with the given file path.
    ///
    /// If the file doesn't exist, it will be created when needed.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the CSV file
    ///
    /// # Returns
    ///
    /// A new `CsvBackend` instance
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use persistent_map::csv::CsvBackend;
    ///
    /// let backend = CsvBackend::new("my_data.csv");
    /// ```
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Ensures the CSV file exists by creating it if it doesn't.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    fn ensure_file_exists(&self) -> std::io::Result<()> {
        if !self.path.exists() {
            // Create parent directories if they don't exist
            if let Some(parent) = self.path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            // Create the file
            std::fs::File::create(&self.path)?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl<K, V> StorageBackend<K, V> for CsvBackend
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
        + std::str::FromStr,
    <K as std::str::FromStr>::Err: std::error::Error + Send + Sync + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + 'static,
{
    async fn load_all(&self) -> Result<HashMap<K, V>, PersistentError> {
        // Ensure the file exists
        self.ensure_file_exists()?;

        // If the file was just created, it's empty, so return an empty HashMap
        if self.path.metadata()?.len() == 0 {
            return Ok(HashMap::new());
        }

        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .from_path(&self.path)
            .map_err(|e| PersistentError::Csv(e.to_string()))?;
        let mut map = HashMap::new();
        for result in rdr.deserialize::<(String, V)>() {
            let (kstr, v) = result.map_err(|e| PersistentError::Csv(e.to_string()))?;
            let key = kstr.parse::<K>().map_err(|_| {
                PersistentError::Serde(serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid key",
                )))
            })?;
            map.insert(key, v);
        }
        Ok(map)
    }

    async fn save(&self, key: K, value: V) -> Result<(), PersistentError> {
        // Ensure the file exists
        self.ensure_file_exists()?;

        let file = OpenOptions::new().append(true).open(&self.path)?;

        let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);

        wtr.serialize((key.to_string(), value))
            .map_err(|e| PersistentError::Csv(e.to_string()))?;

        wtr.flush()?;
        Ok(())
    }

    async fn delete(&self, key: &K) -> Result<(), PersistentError> {
        let mut all: HashMap<K, V> = self.load_all().await?;
        all.remove(key);

        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        let mut wtr = WriterBuilder::new().has_headers(false).from_writer(file);

        for (k, v) in all {
            wtr.serialize((k.to_string(), v))
                .map_err(|e| PersistentError::Csv(e.to_string()))?;
        }

        wtr.flush()?;
        Ok(())
    }
}
