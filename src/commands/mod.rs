pub mod ls;
pub mod cat;
pub mod sys;
pub mod find;

use std::env;
use std::io::{self, Write};
use std::path::Path;
use rustyline::history::DefaultHistory;

pub enum CommandResult {
    Handled(i32),
    NotHandled,
    Exit,
}

/// Parses the line and runs the command if it's a built-in.
pub fn run_builtin(line: &str, history: &DefaultHistory) -> CommandResult {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return CommandResult::Handled(0);
    }

    let args = match parse_args(trimmed) {
        Some(a) => a,
        None => {
            eprintln!("neoterminal: mismatched quotes");
            return CommandResult::Handled(1);
        }
    };

    if args.is_empty() {
        return CommandResult::Handled(0);
    }

    let cmd = args[0].as_str();
    let cmd_args = &args[1..];

    match cmd {
        "exit" => CommandResult::Exit,
        "clear" => {
            print!("\x1b[2J\x1b[1;1H");
            let _ = io::stdout().flush();
            CommandResult::Handled(0)
        }
        "cd" => {
            let target = if cmd_args.is_empty() {
                dirs::home_dir().unwrap_or_else(|| Path::new(".").to_path_buf())
            } else {
                let path_str = cmd_args[0].as_str();
                match path_str {
                    "~" => dirs::home_dir().unwrap_or_else(|| Path::new(".").to_path_buf()),
                    "-" => {
                        if let Ok(oldpwd) = env::var("OLDPWD") {
                            Path::new(&oldpwd).to_path_buf()
                        } else {
                            eprintln!("cd: OLDPWD not set");
                            return CommandResult::Handled(1);
                        }
                    }
                    _ => Path::new(path_str).to_path_buf(),
                }
            };

            let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
            if let Err(e) = env::set_current_dir(&target) {
                eprintln!("cd: {}: {}", target.display(), e);
                CommandResult::Handled(1)
            } else {
                unsafe {
                    env::set_var("OLDPWD", current_dir);
                }
                CommandResult::Handled(0)
            }
        }
        "ls" => {
            let status = ls::run(cmd_args);
            CommandResult::Handled(status)
        }
        "cat" | "view" => {
            if cmd_args.is_empty() {
                eprintln!("usage: {} <file>", cmd);
                CommandResult::Handled(1)
            } else {
                let status = cat::run(&cmd_args[0]);
                CommandResult::Handled(status)
            }
        }
        "sys" => {
            let status = sys::run();
            CommandResult::Handled(status)
        }
        "find" => {
            if cmd_args.is_empty() {
                eprintln!("usage: find <pattern>");
                CommandResult::Handled(1)
            } else {
                let status = find::run(&cmd_args[0]);
                CommandResult::Handled(status)
            }
        }
        "history" => {
            for (idx, entry) in history.iter().enumerate() {
                println!("{:5}  {}", idx + 1, entry);
            }
            CommandResult::Handled(0)
        }
        "help" => {
            print_help();
            CommandResult::Handled(0)
        }
        _ => CommandResult::NotHandled,
    }
}

/// Helper function to split command line arguments, respecting quotes
fn parse_args(line: &str) -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_single_quotes => {
                in_double_quotes = !in_double_quotes;
            }
            '\'' if !in_double_quotes => {
                in_single_quotes = !in_single_quotes;
            }
            ' ' if !in_double_quotes && !in_single_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if in_double_quotes || in_single_quotes {
        return None; // Mismatched quotes
    }

    if !current.is_empty() {
        args.push(current);
    }

    Some(args)
}

fn print_help() {
    println!("\x1b[1;36mNeoTerminal v0.1.0 - Classy Shell Wrapper\x1b[0m");
    println!("\x1b[1mBuilt-in commands:\x1b[0m");
    println!("  \x1b[1;32mls [options]\x1b[0m      Enhanced file listing with icons and sizes.");
    println!("                  Options: -a (hidden files), -l (long format),");
    println!("                           -t (sort by time), -s (sort by size).");
    println!("  \x1b[1;32mcd [dir]\x1b[0m          Change directory (~ for home, - for previous).");
    println!("  \x1b[1;32mcat/view <file>\x1b[0m   View a file with custom syntax highlighting.");
    println!("  \x1b[1;32msys\x1b[0m               Display classy system dashboard (CPU, RAM, Uptime).");
    println!("  \x1b[1;32mfind <pattern>\x1b[0m   Search for files recursively with icons.");
    println!("  \x1b[1;32mhistory\x1b[0m           Show command history.");
    println!("  \x1b[1;32mclear\x1b[0m             Clear the terminal screen.");
    println!("  \x1b[1;32mhelp\x1b[0m              Display this help details.");
    println!("  \x1b[1;32mexit\x1b[0m              Exit NeoTerminal.");
    println!("\nAny other command will be forwarded directly to the system shell.");
}
