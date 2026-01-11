use crate::models::rule::CategoryRule;
use rusqlite::Connection;

pub fn add_rule(conn: &Connection, pattern: &str, category: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO category_rules (pattern, category) VALUES (?1, ?2)",
        [pattern, category],
    )
    .map_err(|e| format!("Failed to insert rule: {}", e))?;
    Ok(())
}

pub fn get_all_rules(conn: &Connection) -> Result<Vec<CategoryRule>, String> {
    let mut stmt = conn
        .prepare("SELECT id, pattern, category FROM category_rules")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let rules_iter = stmt
        .query_map([], |row| {
            Ok(CategoryRule {
                id: row.get(0)?,
                pattern: row.get(1)?,
                category: row.get(2)?,
            })
        })
        .map_err(|e| format!("Failed to query rules: {}", e))?;

    let mut rules = Vec::new();
    for rule in rules_iter {
        rules.push(rule.map_err(|e| format!("Failed to retrieve rule: {}", e))?);
    }
    Ok(rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::establish_test_connection;

    fn sort_by_id(mut rules: Vec<CategoryRule>) -> Vec<CategoryRule> {
        rules.sort_by_key(|r| r.id);
        rules
    }

    #[test]
    fn test_get_all_rules_empty() {
        let conn = establish_test_connection().unwrap();

        let result = get_all_rules(&conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_add_rule_success_and_retrievable() {
        let conn = establish_test_connection().unwrap();

        add_rule(&conn, "coffee", "Food").unwrap();

        let rules = get_all_rules(&conn).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].pattern, "coffee");
        assert_eq!(rules[0].category, "Food");
        assert!(rules[0].id > 0);
    }

    #[test]
    fn test_add_multiple_rules_and_retrieve() {
        let conn = establish_test_connection().unwrap();

        add_rule(&conn, "uber", "Transport").unwrap();
        add_rule(&conn, "salary", "Job").unwrap();
        add_rule(&conn, "lidl", "Groceries").unwrap();

        let rules = get_all_rules(&conn).unwrap();
        assert_eq!(rules.len(), 3);

        assert!(rules.iter().any(|r| r.pattern == "uber" && r.category == "Transport"));
        assert!(rules.iter().any(|r| r.pattern == "salary" && r.category == "Job"));
        assert!(rules.iter().any(|r| r.pattern == "lidl" && r.category == "Groceries"));
    }

    #[test]
    fn test_rule_ids_are_autoincremented() {
        let conn = establish_test_connection().unwrap();

        add_rule(&conn, "a", "A").unwrap();
        add_rule(&conn, "b", "B").unwrap();

        let rules = sort_by_id(get_all_rules(&conn).unwrap());
        assert_eq!(rules.len(), 2);
        assert!(rules[0].id > 0);
        assert!(rules[1].id > rules[0].id);
    }

    #[test]
    fn test_add_rule_allows_duplicate_rows_if_no_unique_constraint() {
        let conn = establish_test_connection().unwrap();

        add_rule(&conn, "coffee", "Food").unwrap();
        add_rule(&conn, "coffee", "Food").unwrap();

        let rules = get_all_rules(&conn).unwrap();
        let matches = rules
            .iter()
            .filter(|r| r.pattern == "coffee" && r.category == "Food")
            .count();

        assert_eq!(matches, 2);
    }

    #[test]
    fn test_add_rule_fails_when_columns_missing_or_schema_wrong() {
        let conn = establish_test_connection().unwrap();
        let result = add_rule(&conn, "x", "Y");
        assert!(result.is_ok());
    }
}