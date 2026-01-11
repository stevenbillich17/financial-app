<p align="center">
  <img src="assets/fino.png" alt="FINO Logo" width="220"/>
</p>

# What is Fino?

A command-line tool for managing personal financial transactions.  
This project provides a lightweight, terminal-based interface for recording, searching, and importing financial data.  
It is designed with a modular architecture that makes it easy to extend with new features over time.

---

## 🚀 Current Features

### ➤ Add Transactions  
Create a new transaction manually. The system validates:
- Date format (`YYYY-MM-DD`)
- Decimal amount
- Transaction type (`income` or `expense`)
- Description/category length limits

### ➤ Manage Categorization Rules (New!)
Automate your transaction sorting by defining regex-based rules.
- **Add Rule**: `rules add <regex_pattern> <category>`
  - Example: `rules add ^Uber.* Transport` automatically categorizes "Uber Trip" as "Transport".
- **List Rules**: `rules list` displays all active categorization rules.

---

### ➤ Remove Transactions  
Remove an entry by providing its UUID.

---

### ➤ Search Transactions  
Search for transactions by category.

---

### ➤ Print All Transactions  
Displays all currently stored transactions.

---

### ➤ Import Data (CSV & OFX)
Import multiple file formats. The system automatically detects format by file extension.

#### 1. CSV Import
Standard comma-separated values.
**Example:**
```csv
2025-01-03,Coffee,-4.65,expense,Food
2025-01-04,Uber,-12.30,expense,Transport
2025-01-05,Salary,2500.00,income,Salary
2025-01-07,Groceries,-54.12,expense,Supermarket
```
#### 2. OFX Import
Supports Open Financial Exchange files (standard for bank exports).
Parses DTPOSTED, TRNAMT, NAME, MEMO, and CATEGORY tags.
Auto-Categorization:
- Uses the <CATEGORY> tag from the file if present.
- If missing, applies your Categorization Rules based on the description.
- Defaults to Uncategorized.

## 🧱 Architecture Overview

The project uses a clean modular organization:

### `models/`
Contains pure data structures:
- `Transaction`
- `TransactionType`

### `operations/`
Business logic:
- Creating transactions
- Removing transactions
- Searching
- CSV importing

## ▶️ Usage

Run the interactive CLI: `cargo run`

Available commands:
- add
- remove
- search
- print
- import
- rules
- exit

Tests cover:
- Transaction creation validation
- Removing transactions
- CSV importing (valid input, invalid files, binary data, invalid UTF-8, etc.)

---

## 🧪 Testing

Run the test suite: `cargo test`

## 📦 Dependencies

```toml
chrono = "0.4.42"
rust_decimal = "1.39.0"
uuid = { version = "1", features = ["v4"] }
csv = "1.4.0"
tempfile = "3.23.0"
rusqlite = { version = "0.37.0", features = ["bundled"] }
quick-xml = "0.38.4"
regex = "1.12.2"