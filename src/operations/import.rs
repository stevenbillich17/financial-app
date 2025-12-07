use super::add::create_transaction;
use crate::db::repository;
use crate::models::transaction::Transaction;
use rusqlite::Connection;
use std::fs::File;

#[derive(Debug)]
pub enum ImportFormat {
    CSV,
}

pub fn import_transactions_to_db(
    conn: &Connection,
    format: ImportFormat,
    path: &str,
) -> Result<usize, String> {
    let transactions = match format {
        ImportFormat::CSV => import_csv(path)?,
        _ => return Err("Unsupported import format".to_string()),
    };
    let mut count = 0;
    for transaction in transactions {
        repository::add_transaction(conn, &transaction)?;
        count += 1;
    }
    Ok(count)
}

fn import_csv(path: &str) -> Result<Vec<Transaction>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file '{}': {}", path, e))?;

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(false)
        .from_reader(file);

    let mut transactions = Vec::new();

    for (line_index, result) in reader.records().enumerate() {
        let record =
            result.map_err(|e| format!("CSV parse error on line {}: {}", line_index + 1, e))?;

        if record.len() != 5 {
            return Err(format!(
                "Invalid number of columns on line {}: expected 5, got {}",
                line_index + 1,
                record.len()
            ));
        }

        let date = record.get(0).unwrap_or("");
        let description = record.get(1).unwrap_or("");
        let amount = record.get(2).unwrap_or("");
        let transaction_type = record.get(3).unwrap_or("");
        let category = record.get(4).unwrap_or("");

        let raw_input = format!(
            "{},{},{},{},{}",
            date, description, amount, transaction_type, category
        );

        let transaction = create_transaction(&raw_input)
            .map_err(|e| format!("Line {}: {}", line_index + 1, e))?;

        transactions.push(transaction);
    }

    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::connection::establish_test_connection;
    use std::io::Write;
    use tempfile::{NamedTempFile};

    fn write_temp_csv(contents: &str) -> NamedTempFile {
        let mut tmp = NamedTempFile::new().expect("Failed to create temp file");
        write!(tmp, "{}", contents).expect("Failed to write test CSV");
        tmp
    }

    #[test]
    fn test_import_csv_to_db_success() {
        let conn = establish_test_connection().unwrap();
        let csv_data = "\
2025-11-10,Salary,1500.00,income,Job
2025-11-11,Coffee,3.50,expense,Food
";

        let tmp = write_temp_csv(csv_data);
        let result = import_transactions_to_db(&conn, ImportFormat::CSV, tmp.path().to_str().unwrap());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        let all = crate::db::repository::get_all_transactions(&conn).unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_import_csv_invalid_data() {
        let conn = establish_test_connection().unwrap();
        let csv_data = "\
bad-date,Salary,1500.00,income,Job
";

        let tmp = write_temp_csv(csv_data);
        let result = import_transactions_to_db(&conn, ImportFormat::CSV, tmp.path().to_str().unwrap());
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Line 1"));
        assert!(error.contains("Invalid date"));
    }

    #[test]
    fn test_import_nonexistent_file() {
        let conn = establish_test_connection().unwrap();
        let result = import_transactions_to_db(&conn, ImportFormat::CSV, "nonexistent.csv");
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to open file"));
    }
}