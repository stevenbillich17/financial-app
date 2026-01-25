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

### ➤ Reports (Custom Date Range)
Generate interactive charts for any custom date range.
- **Report**: `report`
  - Then enter a range like: `10.12.2001-15.02.2022`
  - The report shows:
    - Stacked bar chart (bucketed spend over time)
    - Pie chart (category share)
    - Category spend table
  - Press `q` or `Esc` to exit the report UI.

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
- Importing transactions (CSV/OFX)
- Managing categorization rules
- Generating reports (charts + summaries)

## ▶️ Usage

Run the interactive CLI: `cargo run`

Available commands:
- add
- remove
- search
- print
- import
- rules
- report
- exit

Tests cover:
- Transaction creation validation
- Removing transactions
- CSV importing (valid input, invalid files, binary data, invalid UTF-8, etc.)

---

## 📊 Reporting

The report view summarizes expenses for a custom date range and includes:
- Stacked bar chart of spending over time (bucketed)
- Pie chart showing category share
- Category table with totals

<p align="center">
  <img src="assets/report.png" alt="Report IMG" />
</p>


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
ratatui = "0.30.0"
crossterm = "0.29.0"