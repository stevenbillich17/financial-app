use crate::models::transaction::{Transaction, TransactionType};
use crate::db::{repository, budget_repository, alert_repository};
use rusqlite::Connection;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use uuid::Uuid;

pub fn create_transaction(input: &str) -> Result<Transaction, String> {
    let details_string = input.to_string();
    let details = details_string.trim();
    let detail_parts: Vec<&str> = details.split(',').map(|s| s.trim()).collect();
    
    if detail_parts.len() != 5 {
        return Err(format!(
            "Invalid input format. Expected 5 fields (date,description,amount,type,category), got {}",
            detail_parts.len()
        ));
    }

    let date = NaiveDate::parse_from_str(detail_parts[0], "%Y-%m-%d")
        .map_err(|_| format!("Invalid date format '{}'. Expected YYYY-MM-DD", detail_parts[0]))?;

    let description = detail_parts[1].to_string();
    if description.is_empty() {
        return Err("Description cannot be empty".to_string());
    }

    let amount = detail_parts[2]
        .parse::<Decimal>()
        .map_err(|_| format!("Invalid amount '{}'. Must be a valid number", detail_parts[2]))?;

    let transaction_type = match detail_parts[3].to_lowercase().as_str() {
        "income" => TransactionType::Income,
        "expense" => TransactionType::Expense,
        _ => return Err(format!("Invalid transaction type '{}'. Must be 'income' or 'expense'", detail_parts[3])),
    };

    let category = detail_parts[4].to_string();
    if category.is_empty() {
        return Err("Category cannot be empty".to_string());
    }

    let id = Uuid::new_v4().to_string();

    Ok(Transaction::new(
        id,
        date,
        description,
        amount,
        transaction_type,
        category,
    ))
}

pub fn add_transaction_to_db(conn: &Connection, input: &str) -> Result<(), String> {
    let transaction = create_transaction(input)?;
    repository::add_transaction(conn, &transaction)?;
    check_budget_and_alert(conn, &transaction)?;
    Ok(())
}

pub fn check_budget_and_alert(conn: &Connection, transaction: &Transaction) -> Result<(), String> {
    if transaction.transaction_type != TransactionType::Expense {
        return Ok(());
    }

    if let Some(budget) = budget_repository::get_budget(conn, &transaction.category)? {
        let total = repository::get_total_expenses_by_category(conn, &transaction.category)?;
        if total > budget.amount {
            let message = format!(
                "Budget exceeded for category '{}': budget {}, spent {}",
                budget.category, budget.amount, total
            );
            alert_repository::add_alert(conn, &budget.category, &message)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::establish_test_connection;
    use crate::db::budget_repository;
    use crate::db::alert_repository;
    use rust_decimal::Decimal;

    #[test]
    fn test_create_transaction_valid() {
        let input = "2025-11-10,Salary,1500.00,income,Job";
        let result = create_transaction(input);
        assert!(result.is_ok());
        
        let transaction = result.unwrap();
        assert_eq!(transaction.description, "Salary");
        assert_eq!(transaction.category, "Job");
    }

    #[test]
    fn test_create_transaction_invalid_fields() {
        let input = "2025-11-10,Salary,1500.00,income";
        let result = create_transaction(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected 5 fields"));
    }

    #[test]
    fn test_create_transaction_invalid_date() {
        let input = "invalid-date,Salary,1500.00,income,Job";
        let result = create_transaction(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid date format"));
    }

    #[test]
    fn test_create_transaction_invalid_amount() {
        let input = "2025-11-10,Salary,not-a-number,income,Job";
        let result = create_transaction(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid amount"));
    }

    #[test]
    fn test_create_transaction_invalid_type() {
        let input = "2025-11-10,Salary,1500.00,invalid,Job";
        let result = create_transaction(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid transaction type"));
    }

    #[test]
    fn test_add_transaction_to_db_success() {
        let conn = establish_test_connection().unwrap();
        let input = "2025-11-10,Salary,1500.00,income,Job";
        
        let result = add_transaction_to_db(&conn, input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_transaction_to_db_validation_error() {
        let conn = establish_test_connection().unwrap();
        let input = "invalid-date,Salary,1500.00,income,Job";
        
        let result = add_transaction_to_db(&conn, input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid date format"));
    }

    #[test]
    fn test_budget_alert_generated_on_exceed() {
        let conn = establish_test_connection().unwrap();
        budget_repository::set_budget(&conn, "Food", &Decimal::new(500, 2)).unwrap();

        add_transaction_to_db(&conn, "2025-11-10,Dinner,6.00,expense,Food").unwrap();

        let alerts = alert_repository::get_all_alerts(&conn).unwrap();
        assert_eq!(alerts.len(), 1);
        assert!(alerts[0].message.contains("Budget exceeded"));
    }

    #[test]
    fn test_no_alert_for_income() {
        let conn = establish_test_connection().unwrap();
        budget_repository::set_budget(&conn, "Salary", &Decimal::new(100, 2)).unwrap();

        add_transaction_to_db(&conn, "2025-11-10,Salary,1000.00,income,Salary").unwrap();

        let alerts = alert_repository::get_all_alerts(&conn).unwrap();
        assert!(alerts.is_empty());
    }
}