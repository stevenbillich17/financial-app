# 📊 Financial CLI App (Rust)

A command-line tool for managing personal financial transactions.  
This project provides a lightweight, terminal-based interface for recording, searching, and importing financial data.  
It is designed with a modular architecture that makes it easy to extend with new features over time.

---

## 🚀 Current Features

### ➤ Add Transactions  
Create a new transaction using this format:
All fields are validated:
- Date format (`YYYY-MM-DD`)
- Decimal amount
- Transaction type (`income` or `expense`)
- Description/category length limits

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

### ➤ Import from CSV  
Import transactions from a CSV file in the following format:
Notes:
- The file **does not need an ID column**.
- A new UUID is generated automatically for each imported transaction.
- The importer uses the `csv` crate.
- All imported lines pass through the same validation rules as manual entries.
- The importer handles:
  - Invalid column counts  
  - Incorrect formats  
  - Binary or unreadable files  
  - Invalid UTF-8  

### 📁 Example CSV File
``` 
2025-01-03,Coffee,-4.65,expense,Food
2025-01-04,Uber,-12.30,expense,Transport
2025-01-05,Salary,2500.00,income,Salary
2025-01-07,Groceries,-54.12,expense,Supermarket
```

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

Run the interactive CLI:
Available commands:
- add
- remove
- search
- print
- import
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