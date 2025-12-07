use crate::db::repository;
use crate::models::transaction::Transaction;
use rusqlite::Connection;

pub fn search_transactions_by_category_db(
    conn: &Connection,
    category: &str,
) -> Result<Vec<Transaction>, String> {
    if category.trim().is_empty() {
        return Err("Category cannot be empty".to_string());
    }
    repository::search_by_category(conn, category)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::establish_test_connection;
    use crate::operations::add::add_transaction_to_db;

    #[test]
    fn test_search_transactions_by_category_found() {
        let conn = establish_test_connection().unwrap();
        
        add_transaction_to_db(&conn, "2025-11-10,Coffee,4.50,expense,Food").unwrap();
        add_transaction_to_db(&conn, "2025-11-11,Uber,12.00,expense,Transport").unwrap();
        add_transaction_to_db(&conn, "2025-11-12,Lunch,15.00,expense,Food").unwrap();

        let result = search_transactions_by_category_db(&conn, "Food");
        assert!(result.is_ok());
        
        let transactions = result.unwrap();
        assert_eq!(transactions.len(), 2);
        assert!(transactions.iter().all(|t| t.category == "Food"));
    }

    #[test]
    fn test_search_transactions_by_category_not_found() {
        let conn = establish_test_connection().unwrap();
        
        add_transaction_to_db(&conn, "2025-11-10,Coffee,4.50,expense,Food").unwrap();

        let result = search_transactions_by_category_db(&conn, "Shopping");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_search_transactions_by_category_case_insensitive() {
        let conn = establish_test_connection().unwrap();
        
        add_transaction_to_db(&conn, "2025-11-10,Coffee,4.50,expense,Food").unwrap();

        let result = search_transactions_by_category_db(&conn, "FOOD");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_search_transactions_empty_category() {
        let conn = establish_test_connection().unwrap();
        
        let result = search_transactions_by_category_db(&conn, "");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Category cannot be empty");
    }
}