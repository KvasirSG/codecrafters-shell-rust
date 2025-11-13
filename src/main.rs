#[allow(unused_imports)]
use std::collections::HashMap;
use std::io::{self, Write};
use std::process::{self, Command};
use std::path::Path;
use std::fs;
use std::os::unix::process::CommandExt;

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
    builtins.insert("pwd", pwd_command);
    builtins.insert("cd", cd_command);

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

// Handler for the 'pwd' builtin command
// Prints the full absolute path of the current working directory
fn pwd_command(_args: &[&str]) -> bool {
    match std::env::current_dir() {
        Ok(path) => {
            // Print the absolute path as a string
            if let Some(path_str) = path.to_str() {
                println!("{}", path_str);
            } else {
                println!("Error: current directory path is not valid UTF-8");
            }
            true
        }
        Err(e) => {
            println!("pwd: error retrieving current directory: {}", e);
            true
        }
    }
}

// Handler for the 'cd' builtin command
// Changes the current working directory to the specified path
fn cd_command(args: &[&str]) -> bool {
    // Step 1: Check if a path argument was provided
    if args.len() < 2 {
        println!("cd: missing operand");
        return true;
    }

    // Step 2: Get the path from the arguments (args[1] is the path)
    let path = args[1];

    // Step 3: Try to change to that directory
    match std::env::set_current_dir(path) {
        Ok(_) => {
            // Success! Directory was changed
            true
        }
        Err(_) => {
            // Failed to change directory - print error message
            println!("cd: {}: No such file or directory", path);
            true
        }
    }
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

// Helper function to execute an external program
// Takes the program name and all arguments (including the program name as the first arg)
fn execute_external_program(program: &str, args: &[&str]) -> bool {
    // Try to find the executable in PATH
    if let Some(executable_path) = find_executable_in_path(program) {
        // Execute the program with all arguments
        let mut cmd = Command::new(&executable_path);

        #[cfg(unix)]
        {
            // On Unix, use arg0 to set argv[0] to the original program name
            cmd.arg0(program);
        }

        // Add all remaining arguments (argv[1..])
        for arg in &args[1..] {
            cmd.arg(arg);
        }

        // Execute and wait for the program to complete
        match cmd.status() {
            Ok(_status) => {
                // Program executed successfully
                true
            }
            Err(e) => {
                // Failed to execute the program
                println!("Error executing {}: {}", program, e);
                true
            }
        }
    } else {
        // Program not found in PATH
        println!("{}: command not found", program);
        true
    }
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

// Reads a single command line from stdin
// Returns Some(command) if a line was read, None if EOF was reached
fn read_command_line() -> Option<String> {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut command = String::new();
    match io::stdin().read_line(&mut command) {
        Ok(bytes_read) if bytes_read > 0 => Some(command),
        _ => None,
    }
}

// Parses a command line into command name and arguments
// Returns a Vec of &str where the first element is the command name
fn parse_command(command: &str) -> Vec<&str> {
    command.trim().split_whitespace().collect()
}

// Executes a command (either builtin or external)
// Takes the builtins registry and the parsed command parts
fn execute_command(builtins: &HashMap<&str, CommandHandler>, parts: &[&str]) {
    if let Some(handler) = builtins.get(parts[0]) {
        // Found a builtin command - call its handler function
        handler(parts);
    } else {
        // Not a builtin - try to execute as an external program
        execute_external_program(parts[0], parts);
    }
}

// Main shell loop - continuously reads and executes commands
fn run_shell() {
    // Load all builtin commands into memory at startup
    let builtins = register_builtins();

    // Main shell loop - continuously read and execute commands
    loop {
        // Read user input
        let command = match read_command_line() {
            Some(cmd) => cmd,
            None => break, // EOF reached
        };

        // Parse the command into parts
        let parts = parse_command(&command);

        // Skip empty commands (user just pressed Enter)
        if parts.is_empty() {
            continue;
        }

        // Execute the command
        execute_command(&builtins, &parts);
    }
}

fn main() {
    run_shell();
}
