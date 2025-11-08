mod models;
mod operations;

use std::io;
use models::transaction::Transaction;
use operations::add::read_user_input_and_create_transaction;

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
                println!("Add command selected. Please enter transaction details in the format: date(YYYY-MM-DD), description, amount, type(income/expense), category");
                let transaction = match read_user_input_and_create_transaction() {
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
