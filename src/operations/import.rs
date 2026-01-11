use super::add::create_transaction;
use crate::db::repository;
use crate::models::transaction::{Transaction, TransactionType};
use chrono::NaiveDate;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use regex::Regex;
use rusqlite::Connection;
use rust_decimal::Decimal;
use std::fs::File;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug)]
pub enum ImportFormat {
    CSV,
    OFX,
}

pub fn import_transactions_to_db(
    conn: &Connection,
    format: ImportFormat,
    path: &str,
) -> Result<usize, String> {
    let mut transactions = match format {
        ImportFormat::CSV => import_csv(path)?,
        ImportFormat::OFX => import_ofx(path)?,
    };

    let rules = crate::db::rule_repository::get_all_rules(conn).unwrap_or_default();
    let compiled_rules: Vec<(Regex, String)> = rules
        .into_iter()
        .filter_map(|r| Regex::new(&r.pattern).ok().map(|re| (re, r.category)))
        .collect();

    let mut count = 0;
    for transaction in &mut transactions {
        if transaction.category == "Uncategorized"
            || transaction.category.is_empty()
            || transaction.category == "null"
        {
            for (re, cat) in &compiled_rules {
                if re.is_match(&transaction.description) {
                    transaction.category = cat.clone();
                    break;
                }
            }
        }

        repository::add_transaction(conn, transaction)?;
        count += 1;
    }
    Ok(count)
}

fn import_ofx(path: &str) -> Result<Vec<Transaction>, String> {
    let mut reader = Reader::from_file(path).map_err(|e| format!("Failed to open OFX file: {}", e))?;
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut transactions = Vec::new();

    let mut inside_transaction = false;
    let mut current_tag = String::new();

    let mut t_type = String::new();
    let mut t_date = String::new();
    let mut t_amount = String::new();
    let mut t_name = String::new();
    let mut t_memo = String::new();
    let mut t_fitid = String::new();
    let mut t_category = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_uppercase();
                if name == "STMTTRN" {
                    inside_transaction = true;
                    t_type.clear();
                    t_date.clear();
                    t_amount.clear();
                    t_name.clear();
                    t_memo.clear();
                    t_fitid.clear();
                    t_category.clear();
                }
                current_tag = name;
            }
            Ok(Event::Text(e)) => {
                if inside_transaction {
                    let text = String::from_utf8_lossy(&e).into_owned();
                    match current_tag.as_str() {
                        "TRNTYPE" => t_type = text,
                        "DTPOSTED" => t_date = text,
                        "TRNAMT" => t_amount = text,
                        "NAME" => t_name = text,
                        "MEMO" => t_memo = text,
                        "FITID" => t_fitid = text,
                        "CATEGORY" => t_category = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_uppercase();
                if name == "STMTTRN" && inside_transaction {
                    inside_transaction = false;

                    if t_date.len() < 8 {
                         return Err(format!("Invalid date format in OFX: {}", t_date));
                    }
                    let date_str = &t_date[0..8]; // Take first 8 chars
                    let date = NaiveDate::parse_from_str(date_str, "%Y%m%d")
                        .map_err(|e| format!("Invalid date format {}: {}", t_date, e))?;

                    let amount_dec = Decimal::from_str(&t_amount)
                        .map_err(|e| format!("Invalid amount {}: {}", t_amount, e))?;

                    let (parsed_type, final_amount) = if amount_dec.is_sign_negative() {
                        (TransactionType::Expense, amount_dec.abs())
                    } else {
                        (TransactionType::Income, amount_dec)
                    };

                    let description = if !t_memo.is_empty() {
                        format!("{} - {}", t_name, t_memo)
                    } else {
                        t_name.clone()
                    };

                    let id = if !t_fitid.is_empty() {
                        t_fitid.clone()
                    } else {
                        Uuid::new_v4().to_string()
                    };

                    let category = if !t_category.is_empty() {
                        t_category.clone()
                    } else {
                        "Uncategorized".to_string()
                    };

                    transactions.push(Transaction::new(
                        id,
                        date,
                        description,
                        final_amount,
                        parsed_type,
                        category,
                    ));
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing XML: {}", e)),
            _ => (),
        }
        buf.clear();
    }

    Ok(transactions)
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
        let final_category = if category.trim().is_empty() {
            "Uncategorized"
        } else {
            category
        };

        let raw_input = format!(
            "{},{},{},{},{}",
            date, description, amount, transaction_type, final_category
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

    #[test]
    fn test_import_ofx_success() {
        let conn = establish_test_connection().unwrap();
        let ofx_data = r#"
<OFX>
  <BANKMSGSRSV1>
    <STMTTRNRS>
      <STMTRS>
        <BANKTRANLIST>
          <STMTTRN>
            <TRNTYPE>DEBIT</TRNTYPE>
            <DTPOSTED>20260111120000</DTPOSTED>
            <TRNAMT>-10.50</TRNAMT>
            <FITID>12345</FITID>
            <NAME>Test Transaction</NAME>
            <MEMO>Memo</MEMO>
          </STMTTRN>
        </BANKTRANLIST>
      </STMTRS>
    </STMTTRNRS>
  </BANKMSGSRSV1>
</OFX>
"#;
        let tmp = write_temp_csv(ofx_data);
        let result = import_transactions_to_db(&conn, ImportFormat::OFX, tmp.path().to_str().unwrap());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let txs = crate::db::repository::get_all_transactions(&conn).unwrap();
        assert_eq!(txs[0].amount, Decimal::new(1050, 2));
        assert_eq!(txs[0].category, "Uncategorized");
    }

    #[test]
    fn test_import_ofx_with_category() {
        let conn = establish_test_connection().unwrap();
        let ofx_data = r#"
<OFX>
  <BANKMSGSRSV1>
    <STMTTRNRS>
      <STMTRS>
        <BANKTRANLIST>
          <STMTTRN>
            <TRNTYPE>DEBIT</TRNTYPE>
            <DTPOSTED>20260111120000</DTPOSTED>
            <TRNAMT>-20.00</TRNAMT>
            <FITID>67890</FITID>
            <NAME>Supermarket</NAME>
            <CATEGORY>Groceries</CATEGORY>
          </STMTTRN>
        </BANKTRANLIST>
      </STMTRS>
    </STMTTRNRS>
  </BANKMSGSRSV1>
</OFX>
"#;
        let tmp = write_temp_csv(ofx_data);
        let result = import_transactions_to_db(&conn, ImportFormat::OFX, tmp.path().to_str().unwrap());

        assert!(result.is_ok());
        
        let txs = crate::db::repository::get_all_transactions(&conn).unwrap();
        assert_eq!(txs[0].category, "Groceries");
    }

    #[test]
    fn test_import_with_rules() {
        let conn = establish_test_connection().unwrap();
        crate::db::rule_repository::add_rule(&conn, "Coffee", "Social").unwrap();

        let csv_data = "2025-11-11,Morning Coffee,3.50,expense,";
        let tmp = write_temp_csv(csv_data);

        let result = import_transactions_to_db(&conn, ImportFormat::CSV, tmp.path().to_str().unwrap());
        assert!(result.is_ok());

        let txs = crate::db::repository::get_all_transactions(&conn).unwrap();
        assert_eq!(txs[0].category, "Social");
    }
}