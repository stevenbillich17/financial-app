mod models;

use chrono::NaiveDate;
use models::transaction::Transaction;
use std::io;

pub enum UserCommands {
    Add,
    Remove,
    Exit,
}

fn main() {
    let mut list: Vec<Transaction> = Vec::new(); // Initialize an empty list of integers

    println!("Welcome to the transaction manager!");

    loop {
        println!("Please enter a command (add, remove, exit):");

        // read user input
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let command = check_for_command(parts[0]);
        match command {
            UserCommands::Add => {
                println!(
                    "Add command selected. Provide the transaction details (YYYY-MM-DD, description, amount, Income/Expense, category):"
                );
                let mut details = String::new();
                // read transaction and close the program if things do not match (enough elements) or if the parsing fails
                io::stdin()
                    .read_line(&mut details)
                    .expect("Failed to read line");
                let details = details.trim();
                let detail_parts: Vec<&str> = details.split(',').map(|s| s.trim()).collect();
                if detail_parts.len() != 5 {
                    println!(
                        "Invalid number of details provided. Expected 5 details separated by commas."
                    );
                    continue;
                }
                // Check if right types for all elements
                let date = match NaiveDate::parse_from_str(detail_parts[0], "%Y-%m-%d") {
                    Ok(parsed_date) => parsed_date,
                    Err(_) => {
                        println!("Invalid date format. Please use YYYY-MM-DD.");
                        continue;
                    }
                };

                let amount = match detail_parts[2].parse::<rust_decimal::Decimal>() {
                    Ok(parsed_amount) => parsed_amount,
                    Err(_) => {
                        println!("Invalid amount format. Please provide a valid decimal number.");
                        continue;
                    }
                };

                let transaction_type = match detail_parts[3].to_lowercase().as_str() {
                    "income" => models::transaction::TransactionType::Income,
                    "expense" => models::transaction::TransactionType::Expense,
                    _ => {
                        println!("Invalid transaction type. Use 'income' or 'expense'.");
                        continue;
                    }
                };

                // Check description to have max 255 characters
                let description = detail_parts[1].to_string();
                if description.len() > 255 {
                    println!("Description too long. Maximum 255 characters allowed.");
                    continue;
                }

                // Check category to have max 50 characters
                let category = detail_parts[4].to_string();
                if category.len() > 50 {
                    println!("Category too long. Maximum 50 characters allowed.");
                    continue;
                }

                // Create the transaction
                let transaction = Transaction::new(
                    list.len() as u64 + 1,
                    date,
                    description,
                    amount,
                    transaction_type,
                    category,
                );
                
                list.push(transaction);

                println!("Transaction added successfully.");
                // debug print the list
                println!("Current Transactions: {:?}", list);
            }
            UserCommands::Remove => {
                println!("Remove command selected. Provide the transaction ID to remove:");
                let mut id_input = String::new();
            }
            UserCommands::Exit => {
                println!("Exiting the application.");
                break;
            }
        }
    }
}

fn check_for_command(input: &str) -> UserCommands {
    match input {
        "add" => UserCommands::Add,
        "remove" => UserCommands::Remove,
        "exit" => UserCommands::Exit,
        _ => {
            println!("No valid command found. Exiting.");
            UserCommands::Exit
        }
    }
}
