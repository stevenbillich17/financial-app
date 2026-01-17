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
