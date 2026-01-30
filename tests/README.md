# Tests note

Most tests in this repo live **next to the code** (inside `src/**/*.rs` under `#[cfg(test)] mod tests { ... }`).

That’s not the most “enterprise/professional” layout, but it was an intentional trade-off:

- **One-person team / speed**: fastest way to iterate without maintaining extra test scaffolding.
- **Better access to internals**: unit tests can exercise module-private helpers without widening APIs just for tests.
- **Context stays local**: the test is right next to the code it verifies.
- **Simple setup**: no extra integration test harness needed for most use-cases (many tests already use the in-memory SQLite connection).

The `tests/` directory is still useful for:
- shared fixtures (example CSV/OFX files)
- lightweight documentation like this note

If the project grows (more contributors / CI / more integration scenarios), the plan would be to move higher-level end-to-end checks into Rust integration tests under `tests/*.rs`, while keeping small unit tests co-located where they add the most value.
