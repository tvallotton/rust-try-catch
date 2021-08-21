# try-catch

This crate provides a macro that enables the familiar `try-catch` syntax of other programming languages.
It can be used to easlily group errors and manage them dynamically by type rather than value.

```rust
use std::*;
use serde_json::Value;
catch! {
    try {
        let data = fs::read_to_string("data.json")?;
        let json: Value = serde_json::from_str(&data)?;
    }
    catch error: io::Error {
        println!("Failed to open the file: {}", error)
    }
    catch json_err: serde_json::Error {
        println!("Failed to serialize data: {}", json_err)
    }
}
```
