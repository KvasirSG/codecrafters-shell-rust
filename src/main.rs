#[allow(unused_imports)]
use std::collections::HashMap;
use std::io::{self, Write};
use std::process;
use std::path::Path;
use std::fs;

// Define a type alias for command handler functions
// Each handler takes a slice of command arguments and returns a bool
type CommandHandler = fn(&[&str]) -> bool;

// Create and return a registry of all available builtin commands
// Maps command names (like "echo", "exit") to their handler functions
fn register_builtins() -> HashMap<&'static str, CommandHandler> {
    let mut builtins: HashMap<&'static str, CommandHandler> = HashMap::new();

    // Add each builtin command and its handler function to the registry
    builtins.insert("echo", echo_command);
    builtins.insert("exit", exit_command);
    builtins.insert("type", type_command);

    builtins
}

// Handler for the 'echo' builtin command
// Prints all arguments (after the command name) joined by spaces
fn echo_command(args: &[&str]) -> bool {
    if args.len() > 1 {
        // Skip the first argument (the command name itself) and print the rest
        println!("{}", args[1..].join(" "));
    } else {
        // If no arguments, just print a blank line
        println!();
    }
    true
}

// Handler for the 'exit' builtin command
// Exits the shell with the specified exit code (default 0 if not provided)
fn exit_command(args: &[&str]) -> bool {
    // Try to parse the second argument as an exit code, default to 1 if invalid
    let exit_code = if args.len() > 1 {
        args[1].parse::<i32>().unwrap_or(1)
    } else {
        // If no exit code provided, use 0 (success)
        0
    };
    process::exit(exit_code);
}

// Helper function to search for an executable in PATH
// Returns Some(path) if found with execute permissions, None otherwise
fn find_executable_in_path(command: &str) -> Option<String> {
    // Get the PATH environment variable
    let path_var = std::env::var("PATH").unwrap_or_default();

    // Split PATH by the OS-specific delimiter
    let delimiter = if cfg!(windows) { ";" } else { ":" };

    // Search each directory in PATH
    for dir in path_var.split(delimiter) {
        let path = Path::new(dir).join(command);

        // Check if the file exists
        if path.exists() {
            // Check if it has execute permissions
            if let Ok(metadata) = fs::metadata(&path) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    // On Unix, check if any execute bit is set
                    if metadata.permissions().mode() & 0o111 != 0 {
                        return path.to_str().map(|s| s.to_string());
                    }
                }
                #[cfg(windows)]
                {
                    // On Windows, if the file exists, it's executable
                    return path.to_str().map(|s| s.to_string());
                }
            }
        }
    }

    None
}

// Handler for the 'type' builtin command
// Tells you what kind of command something is (builtin, external program, or not found)
fn type_command(args: &[&str]) -> bool {
    // Check if the user provided a command name to look up
    if args.len() < 2 {
        println!("type: missing operand");
        return true;
    }

    // Get the command name the user wants to look up
    let cmd = args[1];
    // Get the current registry of builtin commands
    let builtins = register_builtins();

    // Check if the command exists in our builtin registry first
    if builtins.contains_key(cmd) {
        println!("{} is a shell builtin", cmd);
    } else if let Some(executable_path) = find_executable_in_path(cmd) {
        // Found an executable in PATH
        println!("{} is {}", cmd, executable_path);
    } else {
        // Command not found as a builtin or in PATH
        println!("{}: not found", cmd);
    }
    true
}

fn main() {
    // Load all builtin commands into memory at startup
    let builtins = register_builtins();

    // Main shell loop - continuously read and execute commands
    loop {
        // Display the shell prompt
        print!("$ ");
        io::stdout().flush().unwrap();

        // Read the user's input line
        let mut command = String::new();
        let bytes_read = io::stdin().read_line(&mut command).unwrap();

        // Check if we reached EOF (end of input, e.g., Ctrl+D or piped input ends)
        if bytes_read == 0 {
            break;
        }

        // Remove leading/trailing whitespace (including newline) from the input
        let command = command.trim();

        // Split the input into individual words (command and arguments)
        let parts: Vec<&str> = command.split_whitespace().collect();

        // Skip empty commands (user just pressed Enter)
        if parts.is_empty() {
            continue;
        }

        // Try to find and execute the command in our builtin registry
        if let Some(handler) = builtins.get(parts[0]) {
            // Found a builtin command - call its handler function
            handler(&parts);
        } else {
            // Command not found - display error message
            println!("{}: command not found", command);
        }
    }
}
