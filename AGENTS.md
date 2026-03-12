# Guidelines for agents

## Dependencies

Manage dependencies using `cargo` - e.g. `cargo add` to add a dependency. This will ensure exact versions are pinned and latest versions are not automatically used.

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

This project follows the testing diamond pattern. The majority of tests should be integration tests. Use unit tests only for testing internal algorithms or database tests.

## Comitting

### Pre-commit Checklist

Before committing changes on code, tests or dependencies run `mise run test`.

### Signing Off

Commits have to be signed off. ALWAYS use `git commit --signoff`, when committing changes.

### Commit Messages

Use conventional commits for commit messages. Keep them concise and descriptive.

## Coding Rules

### UUID

Prefer UUID v4 for IDs. Use them especially as primary keys in databases.

### Dates and Times

Use the `time` crate for dates and times. Prefer UTC over local time.

### Visibility

Prefer private visibility or `pub(crate)` over public visibility (`pub`). Only expose public APIs that are part of the crate's public interface or are necessary for integration tests.

### Imports

Import structs and enums from a module directly, but not functions. Example:

```rust
use crate::models::{self, Task, TaskStatus};

let task = Task {
    status: TaskStatus::Pending,
};

models::create_task(db, "foo bar");
```

### SQL

ALWAYS use lowercase for SQL keywords!

Use sqlx `query!` macro for type safe queries.

Use raw strings for SQL queries and format SQL queries using multiple lines.
