use crate::db::budget_repository;
use crate::models::budget::CategoryBudget;
use rusqlite::Connection;
use rust_decimal::Decimal;
use std::str::FromStr;

pub fn set_budget_db(conn: &Connection, category: &str, amount_str: &str) -> Result<(), String> {
    let amount = Decimal::from_str(amount_str)
        .map_err(|_| format!("Invalid budget amount '{}'. Must be a valid number", amount_str))?;
    if category.trim().is_empty() {
        return Err("Category cannot be empty".to_string());
    }
    budget_repository::set_budget(conn, category.trim(), &amount)
}

pub fn increase_budget_db(conn: &Connection, category: &str, amount_str: &str) -> Result<(), String> {
    let delta = Decimal::from_str(amount_str)
        .map_err(|_| format!("Invalid budget amount '{}'. Must be a valid number", amount_str))?;
    if category.trim().is_empty() {
        return Err("Category cannot be empty".to_string());
    }
    let current = budget_repository::get_budget(conn, category.trim())?
        .map(|b| b.amount)
        .unwrap_or(Decimal::ZERO);
    let new_amount = current + delta;
    budget_repository::set_budget(conn, category.trim(), &new_amount)
}

pub fn decrease_budget_db(conn: &Connection, category: &str, amount_str: &str) -> Result<(), String> {
    let delta = Decimal::from_str(amount_str)
        .map_err(|_| format!("Invalid budget amount '{}'. Must be a valid number", amount_str))?;
    if category.trim().is_empty() {
        return Err("Category cannot be empty".to_string());
    }
    let current = budget_repository::get_budget(conn, category.trim())?
        .map(|b| b.amount)
        .unwrap_or(Decimal::ZERO);
    let new_amount = current - delta;
    if new_amount < Decimal::ZERO {
        return Err("Budget cannot be negative".to_string());
    }
    budget_repository::set_budget(conn, category.trim(), &new_amount)
}

pub fn list_budgets_db(conn: &Connection) -> Result<Vec<CategoryBudget>, String> {
    budget_repository::get_all_budgets(conn)
}

pub fn delete_budget_db(conn: &Connection, category: &str) -> Result<(), String> {
    if category.trim().is_empty() {
        return Err("Category cannot be empty".to_string());
    }
    budget_repository::delete_budget(conn, category.trim())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::establish_test_connection;

    #[test]
    fn test_set_budget_success() {
        let conn = establish_test_connection().unwrap();
        let result = set_budget_db(&conn, "Food", "100.50");
        assert!(result.is_ok());

        let budgets = list_budgets_db(&conn).unwrap();
        assert_eq!(budgets.len(), 1);
        assert_eq!(budgets[0].category, "Food");
    }

    #[test]
    fn test_set_budget_invalid_amount() {
        let conn = establish_test_connection().unwrap();
        let result = set_budget_db(&conn, "Food", "not-a-number");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid budget amount"));
    }

    #[test]
    fn test_set_budget_empty_category() {
        let conn = establish_test_connection().unwrap();
        let result = set_budget_db(&conn, "", "100");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Category cannot be empty");
    }

    #[test]
    fn test_increase_budget_from_zero() {
        let conn = establish_test_connection().unwrap();
        let result = increase_budget_db(&conn, "Travel", "25.00");
        assert!(result.is_ok());

        let budgets = list_budgets_db(&conn).unwrap();
        assert_eq!(budgets.len(), 1);
        assert_eq!(budgets[0].category, "Travel");
        assert_eq!(budgets[0].amount, Decimal::from_str("25.00").unwrap());
    }

    #[test]
    fn test_increase_budget_existing() {
        let conn = establish_test_connection().unwrap();
        set_budget_db(&conn, "Food", "10").unwrap();

        let result = increase_budget_db(&conn, "Food", "5.25");
        assert!(result.is_ok());

        let budgets = list_budgets_db(&conn).unwrap();
        assert_eq!(budgets.len(), 1);
        assert_eq!(budgets[0].amount, Decimal::from_str("15.25").unwrap());
    }

    #[test]
    fn test_decrease_budget_success() {
        let conn = establish_test_connection().unwrap();
        set_budget_db(&conn, "Food", "20").unwrap();

        let result = decrease_budget_db(&conn, "Food", "7.50");
        assert!(result.is_ok());

        let budgets = list_budgets_db(&conn).unwrap();
        assert_eq!(budgets[0].amount, Decimal::from_str("12.50").unwrap());
    }

    #[test]
    fn test_decrease_budget_negative_error() {
        let conn = establish_test_connection().unwrap();
        set_budget_db(&conn, "Food", "5").unwrap();

        let result = decrease_budget_db(&conn, "Food", "10");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Budget cannot be negative");
    }

    #[test]
    fn test_list_budgets_empty() {
        let conn = establish_test_connection().unwrap();
        let budgets = list_budgets_db(&conn).unwrap();
        assert!(budgets.is_empty());
    }

    #[test]
    fn test_delete_budget_success() {
        let conn = establish_test_connection().unwrap();
        set_budget_db(&conn, "Food", "10").unwrap();

        let result = delete_budget_db(&conn, "Food");
        assert!(result.is_ok());

        let budgets = list_budgets_db(&conn).unwrap();
        assert!(budgets.is_empty());
    }

    #[test]
    fn test_delete_budget_not_found() {
        let conn = establish_test_connection().unwrap();
        let result = delete_budget_db(&conn, "Missing");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
