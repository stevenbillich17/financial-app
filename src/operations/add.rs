use std::io;
use chrono::NaiveDate;
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::models::transaction::{Transaction, TransactionType};

pub fn read_user_input_and_create_transaction() -> Result<Transaction, String> {
    let mut details = String::new();
    io::stdin()
        .read_line(&mut details)
        .expect("Failed to read line");
    let details = details.trim();
    let detail_parts: Vec<&str> = details.split(',').map(|s| s.trim()).collect();
    if detail_parts.len() != 5 {
        return Err(format!("Invalid number of details provided. Expected 5 details separated by commas but got {}", detail_parts.len()));  
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
            return Err(format!("Invalid amount format {}. Please provide a valid decimal number.", detail_parts[2]));
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
