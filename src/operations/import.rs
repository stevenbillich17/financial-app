use super::add::create_transaction;
use crate::models::transaction::Transaction;
use std::fs::File;

#[derive(Debug)]
pub enum ImportFormat {
    CSV,
}

pub fn import_transactions(format: ImportFormat, path: &str) -> Result<Vec<Transaction>, String> {
    match format {
        ImportFormat::CSV => import_csv(path),
        _ => Err("Unsupported import format".to_string()),
    }
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
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    fn write_temp_csv(contents: &str) -> NamedTempFile {
        let mut tmp = NamedTempFile::new().expect("Failed to create temp file");
        write!(tmp, "{}", contents).expect("Failed to write test CSV");
        tmp
    }

    fn write_temp(contents: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("failed to create temp file");
        file.write_all(contents)
            .expect("failed to write to temp file");
        file
    }

    #[test]
    fn test_import_csv_success() {
        let csv_data = "\
2025-11-10,Salary,1500.00,income,Job
2025-11-11,Coffee,-3.50,expense,Food
";

        let tmp = write_temp_csv(csv_data);

        let result = import_csv(tmp.path().to_str().unwrap());

        assert!(result.is_ok());

        let list = result.unwrap();
        assert_eq!(list.len(), 2);

        assert_eq!(list[0].description, "Salary");
        assert_eq!(list[1].category, "Food");

        assert!(!list[0].id.is_empty());
        assert!(!list[1].id.is_empty());
    }

    #[test]
    fn test_import_csv_invalid_column_count() {
        let csv_data = "2025-11-10,Salary,1500.00,income\n";

        let tmp = write_temp_csv(csv_data);

        let result = import_csv(tmp.path().to_str().unwrap());

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Invalid number of columns"));
    }

    #[test]
    fn test_import_csv_invalid_data() {
        let csv_data = "\
bad-date,Salary,1500.00,income,Job
";

        let tmp = write_temp_csv(csv_data);
        let result = import_csv(tmp.path().to_str().unwrap());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Line 1"));
        assert!(error.contains("Invalid date"));
    }

    #[test]
    fn test_import_nonexistent_file() {
        let result = import_csv("this/file/does/not/exist.csv");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Failed to open file"));
    }

    #[test]
    fn test_import_directory_instead_of_file() {
        let dir = TempDir::new().unwrap();
        let result = import_csv(dir.path().to_str().unwrap());
        assert!(result.is_err());
        let error = result.unwrap_err();

        assert!(error.contains("Failed to open file"));
    }

    #[test]
    fn test_import_binary_file() {
        // Simulated binary data (executable-like content)
        let binary_data = vec![
            0x7F, b'E', b'L', b'F', // ELF header
            0x00, 0x00, 0x01, 0x02, // null bytes + random bytes
        ];

        let tmp = write_temp(&binary_data);

        let result = import_csv(tmp.path().to_str().unwrap());

        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(
            error.contains("CSV parse error")
                || error.contains("Invalid number of columns")
                || error.contains("UTF-8"),
            "Unexpected error: {}",
            error
        );
    }

    #[test]
    fn test_import_random_text_not_csv() {
        let tmp = write_temp(b"This is not CSV at all\nJust garbage\n");

        let result = import_csv(tmp.path().to_str().unwrap());

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contains("Invalid number of columns"));
    }

    #[test]
    fn test_import_invalid_utf8_file() {
        let invalid_utf8 = vec![0x80, 0x80, 0x80];

        let tmp = write_temp(&invalid_utf8);

        let result = import_csv(tmp.path().to_str().unwrap());

        assert!(result.is_err());
        let error = result.unwrap_err();

        assert!(
            error.contains("CSV parse error") || error.contains("UTF-8"),
            "Unexpected error: {}",
            error
        );
    }
}
