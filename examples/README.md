# PersistentMap Examples

This directory contains examples demonstrating how to use the `persistent-map` crate.

## Running the Examples

To run an example, use the following command:

```bash
cargo run --example <example_name> --features <required_features>
```

For example, to run the SQLite example:

```bash
cargo run --example sqlite_example --features sqlite
```

## Available Examples

### SQLite Example

This example demonstrates how to use the SQLite backend for persistence.

Required features: `sqlite`

```bash
cargo run --example sqlite_example --features sqlite
```

### CSV Example

This example demonstrates how to use the CSV backend for persistence.

Required features: `csv_backend`

```bash
cargo run --example csv_example --features csv_backend
```

### In-Memory Example

This example demonstrates how to create a custom in-memory backend.

No additional features required.

```bash
cargo run --example in_memory_example
```

## Creating Your Own Backend

You can create your own storage backend by implementing the `StorageBackend` trait. See the in-memory example for a simple implementation.
