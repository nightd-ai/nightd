# Guidelines for agents

## Testing

Use a `test` submodule within the Rust source file for unit tests. Example:

```rust
fn foo() {
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_foo() {
    }
}
```

Use a `tests` directory for integration tests. Keep in mind that integration tests only have access to the public API of the crate. For an API that means starting a server within the integration test and making API calls against it.

This project follows the testing diamond pattern. The majority of tests should be integration tests. Use unit tests only for testing internal algorithms.

## Pre-commit Checklist

Before committing changes on code, tests or dependencies run `mise run test`.
