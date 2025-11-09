use std::io;
use uuid::Uuid;
use crate::models::transaction::Transaction;

pub fn read_user_input_and_remove_transaction(
    transactions: &mut Vec<Transaction>,
) -> Result<(), String> {
    let mut id_input = String::new();
    io::stdin()
        .read_line(&mut id_input)
        .map_err(|_| "Failed to read input.".to_string())?;
    let id_input = id_input.trim();

    if id_input.is_empty() {
        return Err("Transaction ID cannot be empty.".to_string());
    }

    let id = match Uuid::parse_str(id_input) {
        Ok(parsed_id) => parsed_id,
        Err(_) => return Err("Invalid transaction ID format. Please provide a valid UUID.".to_string()),
    };

    if let Some(pos) = transactions.iter().position(|t| t.id == id.to_string()) {
        transactions.remove(pos);
        Ok(())
    } else {
        Err(format!("Transaction with ID {} not found.", id))
    }
}