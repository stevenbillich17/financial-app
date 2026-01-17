use crate::models::alert::BudgetAlert;
use chrono::Utc;
use rusqlite::Connection;

pub fn add_alert(conn: &Connection, category: &str, message: &str) -> Result<i32, String> {
    let created_at = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO budget_alerts (category, message, created_at) VALUES (?1, ?2, ?3)",
        [category, message, &created_at],
    )
    .map_err(|e| format!("Failed to insert alert: {}", e))?;
    Ok(conn.last_insert_rowid() as i32)
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

pub fn get_alerts_after_id(conn: &Connection, last_id: i32) -> Result<Vec<BudgetAlert>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, category, message, created_at FROM budget_alerts WHERE id > ?1 ORDER BY id ASC",
        )
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let iter = stmt
        .query_map([last_id], |row| {
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

pub fn get_alerts_by_ids(conn: &Connection, ids: &[i32]) -> Result<Vec<BudgetAlert>, String> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query = format!(
        "SELECT id, category, message, created_at FROM budget_alerts WHERE id IN ({}) ORDER BY id ASC",
        placeholders
    );

    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let params: Vec<rusqlite::types::Value> = ids.iter().map(|id| (*id).into()).collect();
    let iter = stmt
        .query_map(rusqlite::params_from_iter(params), |row| {
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

pub fn get_last_alert_id(conn: &Connection) -> Result<i32, String> {
    let mut stmt = conn
        .prepare("SELECT IFNULL(MAX(id), 0) FROM budget_alerts")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let last_id: i32 = stmt
        .query_row([], |row| row.get(0))
        .map_err(|e| format!("Failed to get last alert id: {}", e))?;

    Ok(last_id)
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

    #[test]
    fn test_get_alerts_after_id() {
        let conn = establish_test_connection().unwrap();
        add_alert(&conn, "Food", "Budget exceeded").unwrap();
        add_alert(&conn, "Travel", "Budget exceeded again").unwrap();

        let last_id = get_last_alert_id(&conn).unwrap();
        let none = get_alerts_after_id(&conn, last_id).unwrap();
        assert!(none.is_empty());

        let alerts = get_alerts_after_id(&conn, 0).unwrap();
        assert_eq!(alerts.len(), 2);
        assert_eq!(alerts[0].category, "Food");
        assert_eq!(alerts[1].category, "Travel");
    }

    #[test]
    fn test_get_alerts_by_ids() {
        let conn = establish_test_connection().unwrap();
        let id1 = add_alert(&conn, "Food", "Budget exceeded").unwrap();
        let id2 = add_alert(&conn, "Travel", "Budget exceeded again").unwrap();

        let alerts = get_alerts_by_ids(&conn, &[id2, id1]).unwrap();
        assert_eq!(alerts.len(), 2);
        assert_eq!(alerts[0].id, id1);
        assert_eq!(alerts[1].id, id2);
    }
}
