# Testing Guidelines

Instead of test pyramid follow a diamond shaped testing approach.

## Unit Tests

Use them only, if there is concrete logic to be tested like a specific algorithm. Unit tests are placed in a test submodule within the same Rust source file.

Example:

```rust
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}```

## Integration Tests

They are the primary tests in this project and are placed in the tests directory of a crate.

Example:

```
nightd
├── Cargo.lock
├── Cargo.toml
├── src
│   └── lib.rs
└── tests
    └── integration_test.rs
```

Integration tests can only access the public API of the crate.
