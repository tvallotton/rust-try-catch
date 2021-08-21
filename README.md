This crate provides a macro that enables the familiar `try-catch` syntax of other programming languages.
It can be used to easlily group errors and manage them dynamically by type rather than value.

```rust
use try_catch::catch;
use std::*;
use serde_json::Value;

catch! {
    try {
        let number: i32 = "10".parse()?;
        let data = fs::read_to_string("data.json")?;
        let json: Value = serde_json::from_str(&data)?;
    }
    catch error: io::Error {
        println!("Failed to open the file: {}", error)
    }
    catch json_err: serde_json::Error {
        println!("Failed to serialize data: {}", json_err)
    }
    catch err {
        println!("Error of unknown type: {}", err)
    }
};

```

Note, if no wildcard is present then the compiler will warn about unused results.
It can alo be used as an expression:

```rust
// We can guarantee that all errors are catched
// so the type of this expression is `i32`.
// It can be guarantieed because the final catch
// does not specify an Error type.
let number: i32 = catch! {
    try {
        let number: i32 = "10".parse()?;
        number
    } catch error {
        0
    }
};
// we can't know for sure if all possible errors are
// handled so the type is still Result.
let result: Result<i32, _> = catch! {
    try {
        let number: i32 = "10".parse()?;
        number
    } catch error: io::Error {
        0
    }
};
```
