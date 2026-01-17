use crate::models::budget::CategoryBudget;
use rusqlite::Connection;
use rust_decimal::Decimal;
use std::str::FromStr;

pub fn set_budget(conn: &Connection, category: &str, amount: &Decimal) -> Result<(), String> {
    conn.execute(
        "INSERT INTO category_budgets (category, amount) VALUES (?1, ?2)\n         ON CONFLICT(category) DO UPDATE SET amount = excluded.amount",
        [category, &amount.to_string()],
    )
    .map_err(|e| format!("Failed to upsert budget: {}", e))?;
    Ok(())
}

pub fn get_budget(conn: &Connection, category: &str) -> Result<Option<CategoryBudget>, String> {
    let mut stmt = conn
        .prepare("SELECT id, category, amount FROM category_budgets WHERE LOWER(category) = LOWER(?1)")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let mut rows = stmt
        .query([category])
        .map_err(|e| format!("Failed to query budget: {}", e))?;

    if let Some(row) = rows.next().map_err(|e| format!("Failed to read budget: {}", e))? {
        let amount_str: String = row
            .get(2)
            .map_err(|e| format!("Failed to read budget amount: {}", e))?;
        let amount = Decimal::from_str(&amount_str)
            .map_err(|e| format!("Failed to parse budget amount: {}", e))?;

        let id: i32 = row.get(0).map_err(|e| format!("Failed to read budget id: {}", e))?;
        let category: String = row
            .get(1)
            .map_err(|e| format!("Failed to read budget category: {}", e))?;

        Ok(Some(CategoryBudget {
            id,
            category,
            amount,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_all_budgets(conn: &Connection) -> Result<Vec<CategoryBudget>, String> {
    let mut stmt = conn
        .prepare("SELECT id, category, amount FROM category_budgets ORDER BY category ASC")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let iter = stmt
        .query_map([], |row| {
            let amount_str: String = row.get(2)?;
            let amount = Decimal::from_str(&amount_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            Ok(CategoryBudget {
                id: row.get(0)?,
                category: row.get(1)?,
                amount,
            })
        })
        .map_err(|e| format!("Failed to query budgets: {}", e))?;

    let mut budgets = Vec::new();
    for budget in iter {
        budgets.push(budget.map_err(|e| format!("Failed to parse budget: {}", e))?);
    }
    Ok(budgets)
}

pub fn delete_budget(conn: &Connection, category: &str) -> Result<(), String> {
    let rows = conn
        .execute("DELETE FROM category_budgets WHERE LOWER(category) = LOWER(?1)", [category])
        .map_err(|e| format!("Failed to delete budget: {}", e))?;

    if rows == 0 {
        return Err(format!("Budget for category '{}' not found", category));
    }
    Ok(())
}
