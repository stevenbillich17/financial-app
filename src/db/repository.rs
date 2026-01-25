use crate::models::transaction::{Transaction, TransactionType};
use rusqlite::Connection;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::str::FromStr;

pub fn add_transaction(conn: &Connection, transaction: &Transaction) -> Result<(), String> {
    let transaction_type_str = match transaction.transaction_type {
        TransactionType::Income => "income",
        TransactionType::Expense => "expense",
    };
    
    conn.execute(
        "INSERT INTO transactions (id, date, description, amount, transaction_type, category) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            &transaction.id,
            transaction.date.to_string(),
            &transaction.description,
            transaction.amount.to_string(),
            transaction_type_str,
            &transaction.category,
        ],
    )
    .map_err(|e| format!("Failed to insert transaction: {}", e))?;
    
    Ok(())
}

pub fn get_all_transactions(conn: &Connection) -> Result<Vec<Transaction>, String> {
    let mut stmt = conn
        .prepare("SELECT id, date, description, amount, transaction_type, category FROM transactions ORDER BY date DESC")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let transaction_iter = stmt
        .query_map([], |row| {
            let date_str: String = row.get(1)?;
            let amount_str: String = row.get(3)?;
            let transaction_type_str: String = row.get(4)?;

            Ok(Transaction {
                id: row.get(0)?,
                date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                description: row.get(2)?,
                amount: Decimal::from_str(&amount_str)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                transaction_type: match transaction_type_str.to_lowercase().as_str() {
                    "income" => TransactionType::Income,
                    "expense" => TransactionType::Expense,
                    _ => return Err(rusqlite::Error::InvalidParameterName("Invalid transaction type".to_string())),
                },
                category: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query transactions: {}", e))?;

    let mut transactions = Vec::new();
    for transaction in transaction_iter {
        transactions.push(transaction.map_err(|e| format!("Failed to parse transaction: {}", e))?);
    }
    
    Ok(transactions)
}

pub fn remove_transaction(conn: &Connection, id: &str) -> Result<(), String> {
    let rows_affected = conn
        .execute("DELETE FROM transactions WHERE id = ?1", [id])
        .map_err(|e| format!("Failed to delete transaction: {}", e))?;

    if rows_affected == 0 {
        return Err(format!("Transaction with ID {} not found", id));
    }
    
    Ok(())
}

pub fn search_by_category(conn: &Connection, category: &str) -> Result<Vec<Transaction>, String> {
    let mut stmt = conn
        .prepare("SELECT id, date, description, amount, transaction_type, category FROM transactions WHERE LOWER(category) = LOWER(?1)")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let transaction_iter = stmt
        .query_map([category], |row| {
            let date_str: String = row.get(1)?;
            let amount_str: String = row.get(3)?;
            let transaction_type_str: String = row.get(4)?;

            Ok(Transaction {
                id: row.get(0)?,
                date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                description: row.get(2)?,
                amount: Decimal::from_str(&amount_str)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                transaction_type: match transaction_type_str.to_lowercase().as_str() {
                    "income" => TransactionType::Income,
                    "expense" => TransactionType::Expense,
                    _ => return Err(rusqlite::Error::InvalidParameterName("Invalid transaction type".to_string())),
                },
                category: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to search transactions: {}", e))?;
    
    let mut transactions = Vec::new();
    for transaction in transaction_iter {
        transactions.push(transaction.map_err(|e| format!("Failed to parse transaction: {}", e))?);
    }
    
    Ok(transactions)
}

pub fn get_expense_transactions_in_range(
    conn: &Connection,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<Transaction>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, date, description, amount, transaction_type, category \n 
            FROM transactions \n 
            WHERE transaction_type = 'expense' AND date >= ?1 AND date <= ?2 \n 
            ORDER BY date ASC",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let transaction_iter = stmt
        .query_map([start_date.to_string(), end_date.to_string()], |row| {
            let date_str: String = row.get(1)?;
            let description_str: String = row.get(2)?;
            let amount_str: String = row.get(3)?;
            let transaction_type_str: String = row.get(4)?;
            let category_str: String = row.get(5)?;

            Ok(Transaction {
                id: row.get(0)?,
                date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                description: description_str,
                amount: Decimal::from_str(&amount_str)
                    .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?,
                transaction_type: match transaction_type_str.to_lowercase().as_str() {
                    "income" => TransactionType::Income, // Ne asteptam doar la expenses for the moment
                    "expense" => TransactionType::Expense,
                    _ => {
                        return Err(rusqlite::Error::InvalidParameterName(
                            "Invalid transaction type".to_string(),
                        ))
                    }
                },
                category: category_str,
            })
        })
        .map_err(|e| format!("Failed to query transactions: {}", e))?;

    let mut transactions = Vec::new();
    for transaction in transaction_iter {
        transactions.push(transaction.map_err(|e| format!("Failed to parse transaction: {}", e))?);
    }

    Ok(transactions)
}

pub fn get_total_expenses_by_category(conn: &Connection, category: &str) -> Result<Decimal, String> {
    let mut stmt = conn
        .prepare(
            "SELECT IFNULL(SUM(CAST(amount AS REAL)), 0) FROM transactions \n             WHERE LOWER(category) = LOWER(?1) AND transaction_type = 'expense'",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let total: f64 = stmt
        .query_row([category], |row| row.get(0))
        .map_err(|e| format!("Failed to calculate total expenses: {}", e))?;

    Decimal::from_f64(total).ok_or_else(|| "Failed to convert total expenses".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::establish_test_connection;
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    fn create_test_transaction(id: &str, category: &str) -> Transaction {
        Transaction::new(
            id.to_string(),
            NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            "Test Transaction".to_string(),
            Decimal::new(10000, 2),
            TransactionType::Income,
            category.to_string(),
        )
    }

    #[test]
    fn test_add_transaction_success() {
        let conn = establish_test_connection().unwrap();
        let transaction = create_test_transaction(&Uuid::new_v4().to_string(), "Salary");

        let result = add_transaction(&conn, &transaction);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_transaction_duplicate_id() {
        let conn = establish_test_connection().unwrap();
        let id = Uuid::new_v4().to_string();
        let transaction = create_test_transaction(&id, "Salary");

        add_transaction(&conn, &transaction).unwrap();
        let result = add_transaction(&conn, &transaction);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("UNIQUE constraint failed"));
    }

    #[test]
    fn test_get_all_transactions_empty() {
        let conn = establish_test_connection().unwrap();
        
        let result = get_all_transactions(&conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_get_all_transactions_multiple() {
        let conn = establish_test_connection().unwrap();
        
        let tx1 = create_test_transaction(&Uuid::new_v4().to_string(), "Food");
        let tx2 = create_test_transaction(&Uuid::new_v4().to_string(), "Transport");
        
        add_transaction(&conn, &tx1).unwrap();
        add_transaction(&conn, &tx2).unwrap();

        let result = get_all_transactions(&conn);
        assert!(result.is_ok());
        
        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 2);
    }

    #[test]
    fn test_remove_transaction_success() {
        let conn = establish_test_connection().unwrap();
        let id = Uuid::new_v4().to_string();
        let transaction = create_test_transaction(&id, "Salary");

        add_transaction(&conn, &transaction).unwrap();
        
        let result = remove_transaction(&conn, &id);
        assert!(result.is_ok());

        let all = get_all_transactions(&conn).unwrap();
        assert_eq!(all.len(), 0);
    }

    #[test]
    fn test_remove_transaction_not_found() {
        let conn = establish_test_connection().unwrap();
        let non_existent_id = Uuid::new_v4().to_string();

        let result = remove_transaction(&conn, &non_existent_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_search_by_category_found() {
        let conn = establish_test_connection().unwrap();
        
        let tx1 = create_test_transaction(&Uuid::new_v4().to_string(), "Food");
        let tx2 = create_test_transaction(&Uuid::new_v4().to_string(), "Transport");
        let tx3 = create_test_transaction(&Uuid::new_v4().to_string(), "Food");
        
        add_transaction(&conn, &tx1).unwrap();
        add_transaction(&conn, &tx2).unwrap();
        add_transaction(&conn, &tx3).unwrap();

        let result = search_by_category(&conn, "Food");
        assert!(result.is_ok());
        
        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 2);
        assert!(transactions.iter().all(|t| t.category == "Food"));
    }

    #[test]
    fn test_search_by_category_not_found() {
        let conn = establish_test_connection().unwrap();
        
        let tx = create_test_transaction(&Uuid::new_v4().to_string(), "Food");
        add_transaction(&conn, &tx).unwrap();

        let result = search_by_category(&conn, "Shopping");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_search_by_category_case_insensitive() {
        let conn = establish_test_connection().unwrap();
        
        let tx = create_test_transaction(&Uuid::new_v4().to_string(), "Food");
        add_transaction(&conn, &tx).unwrap();

        let result = search_by_category(&conn, "FOOD");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}