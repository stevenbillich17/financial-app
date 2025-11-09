use crate::models::transaction::{Transaction, TransactionType};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

pub fn create_transaction(input: &str) -> Result<Transaction, String> {
    let details_string = input.to_string();
    let details = details_string.trim();
    let detail_parts: Vec<&str> = details.split(',').map(|s| s.trim()).collect();
    if detail_parts.len() != 5 {
        return Err(format!(
            "Invalid number of details provided. Expected 5 details separated by commas but got {}",
            detail_parts.len()
        ));
    }

    // Check if right types for all elements
    let date = match NaiveDate::parse_from_str(detail_parts[0], "%Y-%m-%d") {
        Ok(parsed_date) => parsed_date,
        Err(_) => {
            return Err("Invalid date format. Please use YYYY-MM-DD.".to_string());
        }
    };

    let amount = match detail_parts[2].parse::<Decimal>() {
        Ok(parsed_amount) => parsed_amount,
        Err(_) => {
            return Err(format!(
                "Invalid amount format {}. Please provide a valid decimal number.",
                detail_parts[2]
            ));
        }
    };

    let transaction_type = match detail_parts[3].to_lowercase().as_str() {
        "income" => TransactionType::Income,
        "expense" => TransactionType::Expense,
        _ => {
            return Err("Invalid transaction type. Use 'income' or 'expense'.".to_string());
        }
    };

    // Check description to have max 255 characters
    let description = detail_parts[1].to_string();
    if description.len() > 255 {
        return Err("Description too long".to_string());
    }

    // Check category to have max 50 characters
    let category = detail_parts[4].to_string();
    if category.len() > 50 {
        return Err("Category too long".to_string());
    }

    // Create the transaction
    let id = Uuid::new_v4();
    let id_str = id.to_string(); // Convert UUID to a string if needed
    let transaction = Transaction::new(
        id_str,
        date,
        description,
        amount,
        transaction_type,
        category,
    );

    Ok(transaction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::transaction::{TransactionType};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;

    #[test]
    fn test_create_transaction_success() {
        let input = "2025-11-09,Test Description,100.50,income,Test Category";

        let transaction = create_transaction(input);
        assert!(transaction.is_ok());

        let transaction = transaction.unwrap();
        assert_eq!(
            transaction.date,
            NaiveDate::from_ymd_opt(2025, 11, 9).expect("Invalid date")
        );
        assert_eq!(transaction.description, "Test Description");
        assert_eq!(transaction.amount, Decimal::new(10050, 2));
        assert_eq!(transaction.transaction_type, TransactionType::Income);
        assert_eq!(transaction.category, "Test Category");
    }

    #[test]
    fn test_create_transaction_invalid_date() {
        let input = "invalid-date,Test Description,100.50,income,Test Category";
        let mut details = input.to_string();

        let transaction = create_transaction(&mut details);
        assert!(transaction.is_err());
        assert_eq!(
            transaction.unwrap_err(),
            "Invalid date format. Please use YYYY-MM-DD."
        );
    }
}
