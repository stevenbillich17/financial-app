mod models;
mod operations;
mod db;

use operations::import::import_transactions_to_db;
use operations::remove::remove_transaction_from_db;
use operations::search_by_category::search_transactions_by_category_db;
use std::io;

use crate::operations::add::add_transaction_to_db;

pub enum UserCommands {
    Add,
    Remove,
    Exit,
    Print,
    Search,
    Import,
    Rules,
}

fn main() {
    println!("Welcome to the transaction manager!");
    let conn = db::connection::establish_connection().expect("Failed to connect to the database");

    loop {
        println!("Please enter a command (add, import, remove, search, print, rules, exit):");

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
                println!("Add command selected. Please enter transaction details in the format:\ndate(YYYY-MM-DD), description, amount, type(income/expense), category");
                let input = match read_user_input() {
                    Ok(details) => details,
                    Err(e) => {
                        println!("Error reading input: {}", e);
                        continue;
                    }
                };
                match add_transaction_to_db(&conn, &input) {
                    Ok(_) => {
                        println!("Transaction added successfully!");
                    }
                    Err(e) => {
                        println!("Error adding transaction: {}", e);
                        println!("Please try again.");
                    }
                }
            }
            UserCommands::Import => {
                println!("Import command selected. Please enter the file path to import from (supported formats: .csv, .ofx):");
                let input = match read_user_input() {
                    Ok(details) => details,
                    Err(e) => {
                        println!("Error reading input: {}", e);
                        continue;
                    }
                };
                
                let format = if input.to_lowercase().ends_with(".ofx") {
                    Some(operations::import::ImportFormat::OFX)
                } else if input.to_lowercase().ends_with(".csv") {
                    Some(operations::import::ImportFormat::CSV)
                } else {
                    None
                };

                let format = match format {
                    Some(fmt) => fmt,
                    None => {
                        println!("Unrecognized file format for import. Supported formats are .csv and .ofx.");
                        continue;
                    }
                };

                let import_result = import_transactions_to_db(
                    &conn,
                    format,
                    &input,
                );
                match import_result {
                    Ok(number_of_imported_transactions) => {
                        println!("Successfully imported {} transactions.", number_of_imported_transactions);
                    }
                    Err(err) => println!("Error importing transactions: {}", err),
                }
            }
            UserCommands::Remove => {
                println!("Remove command selected. Provide the transaction ID to remove:");
                let input = match read_user_input() {
                    Ok(details) => details,
                    Err(e) => {
                        println!("Error reading input: {}", e);
                        continue;
                    }
                };
                let remove_result = remove_transaction_from_db(&conn, &input);
                match remove_result {
                    Ok(_) => println!("Transaction removed successfully."),
                    Err(err) => println!("Error: {}", err),
                }
            }
            UserCommands::Print => {
                println!("Current Transactions:");
                let list = db::repository::get_all_transactions(&conn).unwrap_or_else(|_| vec![]);  
                for transaction in &list {
                    println!("{:?}", transaction); // Print each transaction on a new line
                }
            }
            UserCommands::Search => {
                println!("Search command selected. Provide the category to search for:");
                let input = match read_user_input() {
                    Ok(details) => details,
                    Err(e) => {
                        println!("Error reading input: {}", e);
                        continue;
                    }
                };
                let results = search_transactions_by_category_db(&conn, &input);
                let transactions = match results {
                    Ok(transactions) => transactions,
                    Err(err) => {
                        println!("Error searching transactions: {}", err);
                        continue;
                    }
                };
                if transactions.is_empty() {
                    println!("No transactions found for category: {}", input);
                } else {
                    println!("Transactions found for category '{}':", input);
                    for transaction in transactions {
                        println!("{:?}", transaction);
                    }
                }
            }
            UserCommands::Rules => {
                println!("Rules command selected. Enter 'add' to create a new rule or 'list' to view existing rules:");
                let input = match read_user_input() {
                     Ok(details) => details,
                     Err(e) => {
                         println!("Error reading input: {}", e);
                         continue;
                     }
                };
                
                match input.trim() {
                    "add" => {
                        println!("Enter rule details in format: pattern category (e.g., 'Uber Transport')");
                        let rule_input = match read_user_input() {
                            Ok(details) => details,
                            Err(e) => {
                                println!("Error reading rule details: {}", e);
                                continue;
                            }
                        };
                        
                        if let Some((pattern, category)) = rule_input.rsplit_once(' ') {
                             match db::rule_repository::add_rule(&conn, pattern.trim(), category.trim()) {
                                 Ok(_) => println!("Rule added: '{}' -> '{}'", pattern.trim(), category.trim()),
                                 Err(e) => println!("Failed to add rule: {}", e),
                             }
                        } else {
                             println!("Invalid format. Please use: <regex_pattern> <category>");
                        }
                    },
                    "list" => {
                        match db::rule_repository::get_all_rules(&conn) {
                            Ok(rules) => {
                                if rules.is_empty() {
                                    println!("No rules defined.");
                                } else {
                                    println!("Categorization Rules:");
                                    for rule in rules {
                                        println!("ID: {}, Pattern: '{}' -> Category: '{}'", rule.id, rule.pattern, rule.category);
                                    }
                                }
                            },
                            Err(e) => println!("Failed to fetch rules: {}", e),
                        }
                    },
                    _ => println!("Invalid option. Use 'add' or 'list'."),
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
        "print" => UserCommands::Print,
        "import" => UserCommands::Import,
        "search" => UserCommands::Search,
        "rules" => UserCommands::Rules,
        _ => {
            println!("No valid command found. Exiting.");
            UserCommands::Exit
        }
    }
}
