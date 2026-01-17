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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::establish_test_connection;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_set_and_get_budget() {
        let conn = establish_test_connection().unwrap();
        set_budget(&conn, "Food", &Decimal::from_str("100").unwrap()).unwrap();

        let budget = get_budget(&conn, "Food").unwrap().unwrap();
        assert_eq!(budget.category, "Food");
        assert_eq!(budget.amount, Decimal::from_str("100").unwrap());
    }

    #[test]
    fn test_get_budget_missing() {
        let conn = establish_test_connection().unwrap();
        let budget = get_budget(&conn, "Missing").unwrap();
        assert!(budget.is_none());
    }

    #[test]
    fn test_set_budget_overwrites() {
        let conn = establish_test_connection().unwrap();
        set_budget(&conn, "Food", &Decimal::from_str("50").unwrap()).unwrap();
        set_budget(&conn, "Food", &Decimal::from_str("75").unwrap()).unwrap();

        let budget = get_budget(&conn, "Food").unwrap().unwrap();
        assert_eq!(budget.amount, Decimal::from_str("75").unwrap());
    }

    #[test]
    fn test_get_all_budgets() {
        let conn = establish_test_connection().unwrap();
        set_budget(&conn, "Food", &Decimal::from_str("10").unwrap()).unwrap();
        set_budget(&conn, "Travel", &Decimal::from_str("20").unwrap()).unwrap();

        let budgets = get_all_budgets(&conn).unwrap();
        assert_eq!(budgets.len(), 2);
    }

    #[test]
    fn test_delete_budget_success() {
        let conn = establish_test_connection().unwrap();
        set_budget(&conn, "Food", &Decimal::from_str("10").unwrap()).unwrap();

        let result = delete_budget(&conn, "Food");
        assert!(result.is_ok());
        assert!(get_budget(&conn, "Food").unwrap().is_none());
    }

    #[test]
    fn test_delete_budget_not_found() {
        let conn = establish_test_connection().unwrap();
        let result = delete_budget(&conn, "Missing");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
