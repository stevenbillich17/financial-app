use crate::models::alert::BudgetAlert;
use chrono::Utc;
use rusqlite::Connection;

pub fn add_alert(conn: &Connection, category: &str, message: &str) -> Result<(), String> {
    let created_at = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO budget_alerts (category, message, created_at) VALUES (?1, ?2, ?3)",
        [category, message, &created_at],
    )
    .map_err(|e| format!("Failed to insert alert: {}", e))?;
    Ok(())
}

pub fn get_all_alerts(conn: &Connection) -> Result<Vec<BudgetAlert>, String> {
    let mut stmt = conn
        .prepare("SELECT id, category, message, created_at FROM budget_alerts ORDER BY id DESC")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let iter = stmt
        .query_map([], |row| {
            Ok(BudgetAlert {
                id: row.get(0)?,
                category: row.get(1)?,
                message: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|e| format!("Failed to query alerts: {}", e))?;

    let mut alerts = Vec::new();
    for alert in iter {
        alerts.push(alert.map_err(|e| format!("Failed to parse alert: {}", e))?);
    }
    Ok(alerts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::establish_test_connection;

    #[test]
    fn test_add_and_list_alerts() {
        let conn = establish_test_connection().unwrap();
        add_alert(&conn, "Food", "Budget exceeded").unwrap();
        add_alert(&conn, "Travel", "Budget exceeded again").unwrap();

        let alerts = get_all_alerts(&conn).unwrap();
        assert_eq!(alerts.len(), 2);
        assert_eq!(alerts[0].category, "Travel");
        assert_eq!(alerts[1].category, "Food");
    }
}
