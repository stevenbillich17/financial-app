mod models;
mod operations;
mod db;

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process;

use operations::import::import_transactions_to_db;
use operations::remove::remove_transaction_from_db;
use operations::search_by_category::search_transactions_by_category_db;
use operations::budget::{set_budget_db, increase_budget_db, decrease_budget_db, list_budgets_db, delete_budget_db};
use operations::report::run_report;
use operations::browse::run_browse;
use chrono::NaiveDate;
use std::io;

use crate::operations::add::{add_transaction_to_db, add_transaction_to_db_with_id};
use crate::db::alert_repository;

#[derive(Parser, Debug)]
#[command(
    name = "fino",
    about = "A command-line tool for managing personal financial transactions",
    arg_required_else_help = true,
    after_help = "EXAMPLES:\n  fino add --date 2025-01-03 --description \"Coffee\" --amount 4.65 --type expense --category Food\n  fino import --file ./data.csv\n  fino import --file ./data.ofx --format ofx\n  fino report --from 2025-01-01 --to 2025-01-31\n  fino budget set --category Food --amount 250\n  fino budget increase --category Food --amount 25\n  fino budget list\n  fino search --category Food\n  fino browse\n  fino tui\n  fino interactive\n\nNOTES:\n  - Dates accept ISO YYYY-MM-DD (recommended). Report also accepts DD.MM.YYYY.\n  - Errors are printed to stderr; exit code is non-zero on failure."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add(AddArgs),
    Import(ImportArgs),
    Report(ReportArgs),
    Budget(BudgetArgsTop),
    Search(SearchArgs),
    #[command(alias = "tui")]
    Browse,
    Interactive,
    Print,
    Remove(RemoveArgs),
}

#[derive(Args, Debug)]
struct AddArgs {
    #[arg(long)]
    date: String,

    #[arg(long)]
    description: String,

    #[arg(long)]
    amount: String,

    #[arg(long = "type", value_enum)]
    transaction_type: CliTransactionType,

    #[arg(long)]
    category: String,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum CliTransactionType {
    Income,
    Expense,
}

impl CliTransactionType {
    fn as_str(self) -> &'static str {
        match self {
            CliTransactionType::Income => "income",
            CliTransactionType::Expense => "expense",
        }
    }
}

#[derive(Args, Debug)]
struct ImportArgs {
    #[arg(long)]
    file: PathBuf,

    #[arg(long, value_enum)]
    format: Option<CliImportFormat>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum CliImportFormat {
    Csv,
    Ofx,
}

#[derive(Args, Debug)]
struct ReportArgs {
    #[arg(long)]
    from: String,

    #[arg(long)]
    to: String,
}

#[derive(Args, Debug)]
struct SearchArgs {
    #[arg(long)]
    category: String,
}

#[derive(Args, Debug)]
struct RemoveArgs {
    #[arg(long)]
    id: String,
}

#[derive(Args, Debug)]
struct BudgetArgsTop {
    #[command(subcommand)]
    command: BudgetCommand,
}

#[derive(Subcommand, Debug)]
enum BudgetCommand {
    Set(BudgetSetArgs),
    Increase(BudgetChangeArgs),
    Decrease(BudgetChangeArgs),
    Delete(BudgetDeleteArgs),
    List,
}

#[derive(Args, Debug)]
struct BudgetSetArgs {
    #[arg(long)]
    category: String,
    #[arg(long)]
    amount: String,
}

#[derive(Args, Debug)]
struct BudgetChangeArgs {
    #[arg(long)]
    category: String,
    #[arg(long)]
    amount: String,
}

#[derive(Args, Debug)]
struct BudgetDeleteArgs {
    #[arg(long)]
    category: String,
}

pub enum UserCommands {
    Add,
    Remove,
    Exit,
    Print,
    Search,
    Import,
    Rules,
    Budgets,
    Report,
}

fn main() {
    let cli = Cli::parse();

    let conn = match db::connection::establish_connection() {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("Failed to connect to the database: {}", e);
            process::exit(1);
        }
    };

    let exit_code = match run_command(&conn, cli.command) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("{}", message);
            1
        }
    };

    process::exit(exit_code);
}

fn run_command(conn: &rusqlite::Connection, cmd: Commands) -> Result<(), String> {
    match cmd {
        Commands::Add(args) => {
            if args.description.contains(',') {
                return Err("Description must not contain commas (',') because the current parser is comma-separated.".to_string());
            }
            if args.category.contains(',') {
                return Err("Category must not contain commas (',') because the current parser is comma-separated.".to_string());
            }

            let raw_input = format!(
                "{},{},{},{},{}",
                args.date,
                args.description,
                args.amount,
                args.transaction_type.as_str(),
                args.category
            );

            let (transaction_id, alert_id) = add_transaction_to_db_with_id(conn, &raw_input)?;
            println!("Transaction added successfully. ID: {}", transaction_id);
            if let Some(alert_id) = alert_id {
                let alerts = alert_repository::get_alerts_by_ids(conn, &[alert_id]).unwrap_or_default();
                if !alerts.is_empty() {
                    println!("Alerts generated:");
                    for alert in alerts {
                        println!("[{}] {}", alert.category, alert.message);
                    }
                }
            }
            Ok(())
        }
        Commands::Import(args) => {
            let path_str = args
                .file
                .to_str()
                .ok_or_else(|| "Invalid file path (non-UTF8).".to_string())?;

            let format = match args.format {
                Some(CliImportFormat::Csv) => operations::import::ImportFormat::CSV,
                Some(CliImportFormat::Ofx) => operations::import::ImportFormat::OFX,
                None => detect_import_format(path_str)?,
            };

            let (count, alert_ids) = import_transactions_to_db(conn, format, path_str)?;
            println!("Successfully imported {} transactions.", count);
            if !alert_ids.is_empty() {
                let alerts = alert_repository::get_alerts_by_ids(conn, &alert_ids).unwrap_or_default();
                if !alerts.is_empty() {
                    println!("Alerts generated during import:");
                    for alert in alerts {
                        println!("[{}] {}", alert.category, alert.message);
                    }
                }
            }
            Ok(())
        }
        Commands::Report(args) => {
            let start = parse_cli_date(&args.from)?;
            let end = parse_cli_date(&args.to)?;
            run_report(conn, start, end)
        }
        Commands::Budget(budget) => match budget.command {
            BudgetCommand::Set(args) => {
                set_budget_db(conn, &args.category, &args.amount)?;
                println!("Budget set for category '{}'", args.category.trim());
                Ok(())
            }
            BudgetCommand::Increase(args) => {
                increase_budget_db(conn, &args.category, &args.amount)?;
                println!("Budget increased for category '{}'", args.category.trim());
                Ok(())
            }
            BudgetCommand::Decrease(args) => {
                decrease_budget_db(conn, &args.category, &args.amount)?;
                println!("Budget decreased for category '{}'", args.category.trim());
                Ok(())
            }
            BudgetCommand::Delete(args) => {
                delete_budget_db(conn, &args.category)?;
                println!("Budget deleted for category '{}'", args.category.trim());
                Ok(())
            }
            BudgetCommand::List => {
                let budgets = list_budgets_db(conn)?;
                if budgets.is_empty() {
                    println!("No budgets defined.");
                } else {
                    println!("Budgets:");
                    for budget in budgets {
                        println!("Category: {}, Amount: {}", budget.category, budget.amount);
                    }
                }
                Ok(())
            }
        },
        Commands::Search(args) => {
            let transactions = search_transactions_by_category_db(conn, &args.category)?;
            if transactions.is_empty() {
                println!("No transactions found for category: {}", args.category);
            } else {
                println!("Transactions found for category '{}':", args.category);
                for transaction in transactions {
                    println!("{:?}", transaction);
                }
            }
            Ok(())
        }
        Commands::Browse => run_browse(conn),
        Commands::Interactive => {
            println!("Welcome to FINO interactive mode!");
            run_interactive(conn);
            Ok(())
        }
        Commands::Print => {
            println!("Current Transactions:");
            let list = db::repository::get_all_transactions(conn).unwrap_or_else(|_| vec![]);
            for transaction in &list {
                println!("{:?}", transaction);
            }
            Ok(())
        }
        Commands::Remove(args) => {
            remove_transaction_from_db(conn, &args.id)?;
            println!("Transaction removed successfully.");
            Ok(())
        }
    }
}

fn detect_import_format(path: &str) -> Result<operations::import::ImportFormat, String> {
    let lower = path.to_lowercase();
    if lower.ends_with(".ofx") {
        Ok(operations::import::ImportFormat::OFX)
    } else if lower.ends_with(".csv") {
        Ok(operations::import::ImportFormat::CSV)
    } else {
        Err("Unrecognized file format. Use --format csv|ofx or provide a .csv/.ofx file.".to_string())
    }
}

fn parse_cli_date(input: &str) -> Result<NaiveDate, String> {
    let s = input.trim();
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(s, "%d.%m.%Y"))
        .map_err(|_| format!("Invalid date '{}'. Use YYYY-MM-DD (recommended) or DD.MM.YYYY.", s))
}

fn run_interactive(conn: &rusqlite::Connection) {
    loop {
        println!("Please enter a command (add, import, remove, search, print, rules, budgets, report, exit):");

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
                match add_transaction_to_db(conn, &input) {
                    Ok(alert_id) => {
                        println!("Transaction added successfully!");
                        if let Some(alert_id) = alert_id {
                            println!("Alerts generated:");
                            let alerts = alert_repository::get_alerts_by_ids(conn, &[alert_id]).unwrap_or_default();
                            for alert in alerts {
                                println!("[{}] {}", alert.category, alert.message);
                            }
                        }
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

                let import_result = import_transactions_to_db(conn, format, &input);
                match import_result {
                    Ok((number_of_imported_transactions, alert_ids)) => {
                        println!("Successfully imported {} transactions.", number_of_imported_transactions);
                        if !alert_ids.is_empty() {
                            println!("Alerts generated during import:");
                            let alerts = alert_repository::get_alerts_by_ids(conn, &alert_ids).unwrap_or_default();
                            for alert in alerts {
                                println!("[{}] {}", alert.category, alert.message);
                            }
                        }
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
                let remove_result = remove_transaction_from_db(conn, &input);
                match remove_result {
                    Ok(_) => println!("Transaction removed successfully."),
                    Err(err) => println!("Error: {}", err),
                }
            }
            UserCommands::Print => {
                println!("Current Transactions:");
                let list = db::repository::get_all_transactions(conn).unwrap_or_else(|_| vec![]);
                for transaction in &list {
                    println!("{:?}", transaction);
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
                let results = search_transactions_by_category_db(conn, &input);
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
                            match db::rule_repository::add_rule(conn, pattern.trim(), category.trim()) {
                                Ok(_) => println!("Rule added: '{}' -> '{}'", pattern.trim(), category.trim()),
                                Err(e) => println!("Failed to add rule: {}", e),
                            }
                        } else {
                            println!("Invalid format. Please use: <regex_pattern> <category>");
                        }
                    }
                    "list" => match db::rule_repository::get_all_rules(conn) {
                        Ok(rules) => {
                            if rules.is_empty() {
                                println!("No rules defined.");
                            } else {
                                println!("Categorization Rules:");
                                for rule in rules {
                                    println!(
                                        "ID: {}, Pattern: '{}' -> Category: '{}'",
                                        rule.id, rule.pattern, rule.category
                                    );
                                }
                            }
                        }
                        Err(e) => println!("Failed to fetch rules: {}", e),
                    },
                    _ => println!("Invalid option. Use 'add' or 'list'."),
                }
            }
            UserCommands::Budgets => {
                println!("Budgets command selected. Options: set, increase, decrease, delete, list, back");
                let input = match read_user_input() {
                    Ok(details) => details,
                    Err(e) => {
                        println!("Error reading input: {}", e);
                        continue;
                    }
                };

                match input.trim() {
                    "set" => {
                        println!("Enter budget details in format: category,amount");
                        let budget_input = match read_user_input() {
                            Ok(details) => details,
                            Err(e) => {
                                println!("Error reading budget details: {}", e);
                                continue;
                            }
                        };
                        let parts: Vec<&str> = budget_input.split(',').map(|s| s.trim()).collect();
                        if parts.len() != 2 {
                            println!("Invalid format. Use: category,amount");
                            continue;
                        }
                        match set_budget_db(conn, parts[0], parts[1]) {
                            Ok(_) => println!("Budget set for category '{}'", parts[0]),
                            Err(e) => println!("Failed to set budget: {}", e),
                        }
                    }
                    "increase" => {
                        println!("Enter budget increase in format: category,amount");
                        let budget_input = match read_user_input() {
                            Ok(details) => details,
                            Err(e) => {
                                println!("Error reading budget details: {}", e);
                                continue;
                            }
                        };
                        let parts: Vec<&str> = budget_input.split(',').map(|s| s.trim()).collect();
                        if parts.len() != 2 {
                            println!("Invalid format. Use: category,amount");
                            continue;
                        }
                        match increase_budget_db(conn, parts[0], parts[1]) {
                            Ok(_) => println!("Budget increased for category '{}'", parts[0]),
                            Err(e) => println!("Failed to increase budget: {}", e),
                        }
                    }
                    "decrease" => {
                        println!("Enter budget decrease in format: category,amount");
                        let budget_input = match read_user_input() {
                            Ok(details) => details,
                            Err(e) => {
                                println!("Error reading budget details: {}", e);
                                continue;
                            }
                        };
                        let parts: Vec<&str> = budget_input.split(',').map(|s| s.trim()).collect();
                        if parts.len() != 2 {
                            println!("Invalid format. Use: category,amount");
                            continue;
                        }
                        match decrease_budget_db(conn, parts[0], parts[1]) {
                            Ok(_) => println!("Budget decreased for category '{}'", parts[0]),
                            Err(e) => println!("Failed to decrease budget: {}", e),
                        }
                    }
                    "delete" => {
                        println!("Enter category to delete budget:");
                        let category_input = match read_user_input() {
                            Ok(details) => details,
                            Err(e) => {
                                println!("Error reading category: {}", e);
                                continue;
                            }
                        };
                        match delete_budget_db(conn, &category_input) {
                            Ok(_) => println!("Budget deleted for category '{}'", category_input.trim()),
                            Err(e) => println!("Failed to delete budget: {}", e),
                        }
                    }
                    "list" => match list_budgets_db(conn) {
                        Ok(budgets) => {
                            if budgets.is_empty() {
                                println!("No budgets defined.");
                            } else {
                                println!("Budgets:");
                                for budget in budgets {
                                    println!("Category: {}, Amount: {}", budget.category, budget.amount);
                                }
                            }
                        }
                        Err(e) => println!("Failed to list budgets: {}", e),
                    },
                    "back" => continue,
                    _ => println!("Invalid option. Use set, increase, decrease, delete, list, or back."),
                }
            }
            UserCommands::Report => {
                println!("Report command selected. Enter date range in format: DD.MM.YYYY-DD.MM.YYYY (e.g., 10.12.2001-15.02.2022)");
                let input = match read_user_input() {
                    Ok(details) => details,
                    Err(e) => {
                        println!("Error reading input: {}", e);
                        continue;
                    }
                };

                let (start_str, end_str) = match input.split_once('-') {
                    Some(parts) => parts,
                    None => {
                        println!("Invalid range format. Use DD.MM.YYYY-DD.MM.YYYY.");
                        continue;
                    }
                };

                let start_date = match NaiveDate::parse_from_str(start_str.trim(), "%d.%m.%Y") {
                    Ok(date) => date,
                    Err(_) => {
                        println!("Invalid start date. Use DD.MM.YYYY.");
                        continue;
                    }
                };

                let end_date = match NaiveDate::parse_from_str(end_str.trim(), "%d.%m.%Y") {
                    Ok(date) => date,
                    Err(_) => {
                        println!("Invalid end date. Use DD.MM.YYYY.");
                        continue;
                    }
                };

                if let Err(e) = run_report(conn, start_date, end_date) {
                    println!("Failed to generate report: {}", e);
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
        "budgets" => UserCommands::Budgets,
        "report" => UserCommands::Report,
        _ => {
            println!("No valid command found. Exiting.");
            UserCommands::Exit
        }
    }
}
