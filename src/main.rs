#[allow(unused_imports)]
use std::io::{self, Write};
use std::process;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        let bytes_read = io::stdin().read_line(&mut command).unwrap();

        // Check if we reached EOF (no bytes read)
        if bytes_read == 0 {
            break;
        }

        let command = command.trim();

        // Parse the command
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        // Handle the exit builtin
        if parts[0] == "exit" {
            let exit_code = if parts.len() > 1 {
                parts[1].parse::<i32>().unwrap_or(1)
            } else {
                0
            };
            process::exit(exit_code);
        }

        println!("{}: command not found", command);
    }
}
