use std::io;

fn main() {
    let mut list: Vec<i32> = Vec::new(); // Initialize an empty list of integers

    println!("Welcome! Use 'add <int_value>' to add, 'remove <int_value>' to remove, or 'exit' to quit.");

    loop {
        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let input = input.trim(); // Remove trailing newline and spaces

        // Exit the loop if the user types "exit"
        if input.eq_ignore_ascii_case("exit") {
            println!("Exiting. Final list: {:?}", list);
            break;
        }

        // Split the input into command and value
        let mut parts = input.split_whitespace();
        let command = parts.next();
        let value = parts.next();

        match (command, value) {
            (Some("add"), Some(value)) => {
                if let Ok(num) = value.parse::<i32>() {
                    list.push(num);
                    println!("Added {}. Current list: {:?}", num, list);
                } else {
                    println!("Invalid number: {}", value);
                }
            }
            (Some("remove"), Some(value)) => {
                if let Ok(num) = value.parse::<i32>() {
                    if let Some(pos) = list.iter().position(|&x| x == num) {
                        list.remove(pos);
                        println!("Removed {}. Current list: {:?}", num, list);
                    } else {
                        println!("Value {} not found in the list.", num);
                    }
                } else {
                    println!("Invalid number: {}", value);
                }
            }
            _ => {
                println!("Invalid command. Use 'add <int_value>', 'remove <int_value>', or 'exit'.");
            }
        }
    }
}