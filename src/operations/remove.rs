use uuid::Uuid;
use crate::models::transaction::Transaction;

pub fn read_user_input_and_remove_transaction(
    input: &str,
    transactions: &mut Vec<Transaction>,
) -> Result<(), String> {

    if input.is_empty() {
        return Err("Transaction ID cannot be empty.".to_string());
    }

    let id = match Uuid::parse_str(input) {
        Ok(parsed_id) => parsed_id,
        Err(_) => return Err("Invalid transaction ID format. Please provide a valid UUID.".to_string()),
    };

    if let Some(pos) = transactions.iter().position(|t| t.id == id.to_string()) {
        transactions.remove(pos);
        Ok(())
    } else {
        Err(format!("Transaction with ID {} not found.", id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{Transaction, TransactionType};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    // Helper function to create a test transaction
    fn create_test_transaction(id: &str) -> Transaction {
        Transaction {
            id: id.to_string(),
            date: NaiveDate::from_ymd_opt(2025, 11, 9).expect("Invalid date"),
            description: "Test Description".to_string(),
            amount: Decimal::new(10050, 2),
            transaction_type: TransactionType::Income,
            category: "Test Category".to_string(),
        }
    }

    #[test]
    fn test_remove_transaction_success() {
        let mut transactions = vec![
            create_test_transaction("550e8400-e29b-41d4-a716-446655440000"),
            create_test_transaction("550e8400-e29b-41d4-a716-446655440001"),
        ];

        // Simulate user input for the transaction ID
        let id_to_remove = "550e8400-e29b-41d4-a716-446655440000";
        let result = read_user_input_and_remove_transaction(&id_to_remove, &mut transactions);

        assert!(result.is_ok());
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].id, "550e8400-e29b-41d4-a716-446655440001");
    }

    #[test]
    fn test_remove_transaction_not_found() {
        let mut transactions = vec![
            create_test_transaction("550e8400-e29b-41d4-a716-446655440000"),
        ];

        // Simulate user input for a non-existent transaction ID
        let id_to_remove = "550e8400-e29b-41d4-a716-446655440999";
        let result = read_user_input_and_remove_transaction(&id_to_remove, &mut transactions);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Transaction with ID 550e8400-e29b-41d4-a716-446655440999 not found."
        );
    }

    #[test]
    fn test_remove_transaction_invalid_uuid() {
        let mut transactions = vec![
            create_test_transaction("550e8400-e29b-41d4-a716-446655440000"),
        ];

        // Simulate user input for an invalid UUID
        let id_to_remove = "invalid-uuid";
        let result = read_user_input_and_remove_transaction(&id_to_remove, &mut transactions);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid transaction ID format. Please provide a valid UUID."
        );
    }

    #[test]
    fn test_remove_transaction_empty_input() {
        let mut transactions = vec![
            create_test_transaction("550e8400-e29b-41d4-a716-446655440000"),
        ];

        // Simulate empty user input
        let id_to_remove = "";
        let result = read_user_input_and_remove_transaction(&id_to_remove, &mut transactions);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Transaction ID cannot be empty.");
    }
}