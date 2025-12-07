use crate::db::repository;
use rusqlite::Connection;
use uuid::Uuid;

pub fn remove_transaction_from_db(conn: &Connection, input: &str) -> Result<(), String> {
    if input.is_empty() {
        return Err("Transaction ID cannot be empty.".to_string());
    }
    let id = match Uuid::parse_str(input) {
        Ok(parsed_id) => parsed_id,
        Err(_) => return Err("Invalid transaction ID format. Please provide a valid UUID.".to_string()),
    };
    repository::remove_transaction(conn, &id.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::establish_test_connection;
    use crate::operations::add::add_transaction_to_db;

    #[test]
    fn test_remove_transaction_success() {
        let conn = establish_test_connection().unwrap();
        
        add_transaction_to_db(&conn, "2025-11-10,Salary,1500.00,income,Job").unwrap();
        let transactions = crate::db::repository::get_all_transactions(&conn).unwrap();
        let id = &transactions[0].id;

        let result = remove_transaction_from_db(&conn, id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_remove_transaction_not_found() {
        let conn = establish_test_connection().unwrap();
        let non_existent_id = "550e8400-e29b-41d4-a716-446655440999";
        
        let result = remove_transaction_from_db(&conn, non_existent_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_remove_transaction_invalid_uuid() {
        let conn = establish_test_connection().unwrap();
        let result = remove_transaction_from_db(&conn, "invalid-uuid");
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid transaction ID format. Please provide a valid UUID."
        );
    }

    #[test]
    fn test_remove_transaction_empty_input() {
        let conn = establish_test_connection().unwrap();
        let result = remove_transaction_from_db(&conn, "");
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Transaction ID cannot be empty.");
    }
}