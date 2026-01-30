# Fino — Architecture

## Overview
Fino is a local-first finance tracker built as a Rust CLI.

At a high level:
- **Input** arrives via `clap` subcommands (or the legacy interactive prompt).
- **Operations layer** validates/normalizes inputs and coordinates workflows.
- **Database layer** persists data in SQLite via `rusqlite`.
- **TUI layer** (Ratatui + Crossterm) renders interactive screens like Reports and Browse.

The storage is a single SQLite file created in the working directory: `financial_app.db`.

## Key Modules

### `src/main.rs` (CLI / Composition Root)
Responsibilities:
- Parses CLI arguments (`clap`).
- Establishes SQLite connection.
- Routes subcommands to the relevant operation.
- Formats user-facing output and error messages.

Main subcommands:
- `add`, `import`, `remove`, `search`, `print`
- `budget set|increase|decrease|list|delete`
- `report` (TUI)
- `browse` (TUI, alias: `tui`)
- `interactive` (legacy prompt-driven mode)

### `src/models/` (Domain Types)
Pure data structures used across layers.

Key types:
- `Transaction` and `TransactionType`
- `CategoryBudget`
- `CategoryRule`
- `BudgetAlert`

### `src/operations/` (Use Cases)
Implements the business workflows and validation. Operations typically:
- Parse/validate user-provided strings
- Build domain models
- Call repositories
- Optionally trigger secondary effects (e.g., budget alerts)

Important operations:
- `add`: transaction creation + insert + budget alert check
- `import`: CSV/OFX parsing + categorization + insert + budget alert checks
- `budget`: set/increase/decrease/list/delete budgets
- `search_by_category`: validation + category query
- `report`: loads range data and renders interactive UI
- `browse`: loads transactions and renders interactive filter/sort UI

### `src/db/` (Persistence)
Encapsulates SQLite schema management and queries.

Files:
- `connection.rs`: opens the DB and ensures tables exist
- `repository.rs`: transaction queries/inserts/removals
- `rule_repository.rs`: categorization rule persistence
- `budget_repository.rs`: budget persistence
- `alert_repository.rs`: budget alert persistence

## Database Schema
Created on startup in `db::connection::establish_connection()`.

### `transactions`
- `id TEXT PRIMARY KEY`
- `date TEXT NOT NULL` (stored as ISO date string)
- `description TEXT NOT NULL`
- `amount TEXT NOT NULL` (stored as decimal string)
- `transaction_type TEXT NOT NULL` (`income` | `expense`)
- `category TEXT NOT NULL`

### `category_rules`
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `pattern TEXT NOT NULL` (regex)
- `category TEXT NOT NULL`

### `category_budgets`
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `category TEXT NOT NULL UNIQUE`
- `amount TEXT NOT NULL` (decimal string)

### `budget_alerts`
- `id INTEGER PRIMARY KEY AUTOINCREMENT`
- `category TEXT NOT NULL`
- `message TEXT NOT NULL`
- `created_at TEXT NOT NULL` (RFC3339 timestamp)

## Core Workflows

### 1) Add Transaction
Flow:
1. CLI collects typed fields (`--date`, `--description`, `--amount`, `--type`, `--category`).
2. Operation builds a `Transaction` with a new UUID.
3. Transaction is inserted into `transactions`.
4. If it’s an expense, the system checks the category budget and creates an alert if exceeded.

Budget alert check:
- Reads budget for the transaction category.
- Computes total expenses for that category.
- If `total_spent > budget_amount`, inserts a row into `budget_alerts`.

### 2) Import Transactions (CSV / OFX)
The import operation:
1. Parses input file into a list of `Transaction` values.
2. Loads all categorization rules from `category_rules` and compiles them into `Regex`.
3. For each transaction:
   - If category is `Uncategorized`/empty/`null`, applies the first matching rule based on the transaction **description**.
   - Inserts the transaction.
   - Checks budgets and writes alerts when exceeded.

#### CSV parsing
Expected columns (no header):
`date,description,amount,transaction_type,category`

If category is empty, it becomes `Uncategorized`.

#### OFX parsing
- Reads `DTPOSTED`, `TRNAMT`, `NAME`, `MEMO`, `FITID`, optional `CATEGORY`.
- If `FITID` is present it becomes the transaction id; otherwise a UUID is generated.
- If `CATEGORY` is missing, it becomes `Uncategorized` and rules may apply.

### 3) Report (TUI)
The Report UI is rendered in the terminal alternate screen:
- Loads expense transactions in the requested date range.
- Buckets data by date span (daily/weekly/biweekly depending on range size).
- Shows:
  - stacked bar chart (spend over time)
  - pie chart (category share)
  - category totals table

Keys:
- `q` or `Esc` to exit

### 4) Browse (TUI)
Browse is an interactive transaction viewer:
- Loads all transactions
- Provides filtering (category, type, date range) and sorting
- Shows list and details views in a TUI

## Error Handling
Most operations return `Result<_, String>`:
- On failure, CLI prints a human-readable error to stderr and exits non-zero.
- SQLite errors are wrapped with context at repository boundaries.

## Extensibility Notes
Common extension points:
- Add a new operation in `src/operations/` and wire it in `src/main.rs`.
- Add a new table + repository module in `src/db/`.
- Add new report/browse interactions by extending the Ratatui state machines.
