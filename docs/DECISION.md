
## Why Clap for the CLI?
Clap was chosen for the primary interface because it provides:
- **Ergonomic argument parsing** with derive macros.
- **Built-in help/usage generation** and validation.
- **Composable subcommands** (`budget set|increase|...`) matching the app’s use-cases.

Trade-offs:
- A strict CLI can feel less discoverable than a guided prompt for first-time users.

## Why an `operations/` folder?
`operations/` is the “use-case layer”: it holds workflow code that sits between CLI/UI and persistence.

Reasons:
- **Keeps `main.rs` thin**: `main.rs` routes commands; operations contain logic.
- **Testability**: operations are easy to unit test with an in-memory DB.
- **Separation of concerns**: repositories focus on SQL, operations focus on business rules.

## Why UUID transaction IDs?
Transaction IDs are generated with UUIDv4 (except when importing OFX with a `FITID`).

Reasons:
- **Uniqueness across imports**: avoids collisions when merging data sources.
- **User-friendly for referencing/removal**: stable identifier for `remove --id ...`.
- **OFX compatibility**: preserves bank-provided identifiers when available.

Trade-offs:
- UUIDs are longer than integer IDs.

## Why store Decimal values as TEXT?
Amounts and budgets are represented as `rust_decimal::Decimal` in memory and stored as string (`TEXT`) in SQLite.

Reasons:
- Avoids floating-point rounding issues.
- Preserves the exact decimal representation.

Trade-offs:
- Aggregations in SQL are more limited; some totals are computed via parsing.

## Why regex-based categorization rules?
Rules are stored as regex patterns and applied to transaction descriptions during import.

Reasons:
- **Flexible matching** for messy real-world statements.
- **User-controlled automation** without changing code.
- Rules can be applied when the imported category is missing (`Uncategorized`).

Trade-offs:
- Bad regex patterns can be expensive; invalid patterns are skipped when compiling.
