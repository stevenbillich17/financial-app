use crate::models::transaction::Transaction;

pub fn search_transactions_by_category<'a>(
    category: &str,
    transactions: &'a [Transaction],
) -> Vec<&'a Transaction> {
    transactions
        .iter()
        .filter(|transaction| transaction.category.eq_ignore_ascii_case(category))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{Transaction, TransactionType};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    // Helper function to create a test transaction
    fn create_test_transaction(id: &str, category: &str) -> Transaction {
        Transaction {
            id: id.to_string(),
            date: NaiveDate::from_ymd_opt(2025, 11, 9).expect("Invalid date"),
            description: "Test Description".to_string(),
            amount: Decimal::new(10050, 2),
            transaction_type: TransactionType::Income,
            category: category.to_string(),
        }
    }

    #[test]
    fn test_search_transactions_by_category_found() {
        let transactions = vec![
            create_test_transaction("1", "Food"),
            create_test_transaction("2", "Travel"),
            create_test_transaction("3", "Food"),
        ];

        let result = search_transactions_by_category("Food", &transactions);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "1");
        assert_eq!(result[1].id, "3");
    }

    #[test]
    fn test_search_transactions_by_category_not_found() {
        let transactions = vec![
            create_test_transaction("1", "Food"),
            create_test_transaction("2", "Travel"),
        ];

        let result = search_transactions_by_category("Shopping", &transactions);
        assert!(result.is_empty());
    }

    #[test]
    fn test_search_transactions_by_category_case_insensitive() {
        let transactions = vec![
            create_test_transaction("1", "Food"),
            create_test_transaction("2", "food"),
        ];

        let result = search_transactions_by_category("FOOD", &transactions);
        assert_eq!(result.len(), 2);
    }
}