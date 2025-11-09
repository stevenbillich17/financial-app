mod models;
mod operations;

use models::transaction::Transaction;
use operations::add::create_transaction;
use operations::remove::read_user_input_and_remove_transaction;
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
        let input = match read_user_input() {
            Ok(cmd) => cmd,
            Err(e) => {
                println!("Error reading input: {}", e);
                continue;
            }
        };
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let command = check_for_command(parts[0]);
        match command {
            UserCommands::Add => {
                println!(
                    "Add command selected. Please enter transaction details in the format: date(YYYY-MM-DD), description, amount, type(income/expense), category"
                );
                let mut input = match read_user_input() {
                    Ok(details) => details,
                    Err(e) => {
                        println!("Error reading input: {}", e);
                        continue;
                    }
                };
                let transaction = match create_transaction(&mut input) {
                    Ok(tx) => tx,
                    Err(e) => {
                        println!("Error adding transaction: {}", e);
                        println!("Please try again.");
                        continue;
                    }
                };
                list.push(transaction);
                println!("Transaction added successfully.");
                println!("Current Transactions: {:?}", list);
            }
            UserCommands::Remove => {
                println!("Remove command selected. Provide the transaction ID to remove:");
                let remove_result = read_user_input_and_remove_transaction(&mut list);
                match remove_result {
                    Ok(_) => println!("Transaction removed successfully."),
                    Err(err) => println!("Error: {}", err),
                }
            }
            UserCommands::Exit => {
                println!("Exiting the application.");
                break;
            }
        }
    }
}

fn read_user_input() -> Result<String, String> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|_| "Failed to read line".to_string())?;
    Ok(input.trim().to_string())
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
