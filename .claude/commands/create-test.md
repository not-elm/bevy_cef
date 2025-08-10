---
allowed-tools: Bash(cargo test:*),
description: Create new tests
---

1. Reading $ARGUMENTS and understanding the implementation details.
2. Creating new tests for $ARGUMENTS in `tests` module in the same file.
3. Run `cargo test --workspace --all-features` to ensure all tests pass.
4. If any tests fail, fix the issues and re-run the tests.

## Contracts

- You have to write the rust-doc that describes the test case for each test function.