mod icons;
mod prompt;
mod completer;
mod shell;
mod commands;

use completer::NeoHelper;
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Config, Editor};
use std::path::PathBuf;

fn main() -> rustyline::Result<()> {
    // 0. Handle Ctrl+C signal to prevent parent crash during child command execution
    let _ = ctrlc::set_handler(move || {
        // Do nothing; child processes will receive SIGINT and terminate themselves,
        // and rustyline will override this handler when readline is active.
    });

    // 1. Print a beautiful greeting banner
    print_greeting();

    // 2. Setup rustyline editor with our custom helper
    let config = Config::builder()
        .auto_add_history(true)
        .color_mode(rustyline::config::ColorMode::Forced)
        .build();
    
    let mut editor: Editor<NeoHelper, DefaultHistory> = Editor::with_config(config)?;
    editor.set_helper(Some(NeoHelper::new()));

    // 3. Load command history from user's home directory
    let history_path = get_history_path();
    if let Some(ref path) = history_path {
        if path.exists() {
            let _ = editor.load_history(path);
        }
    }

    let mut last_exit_status = 0;

    // 4. Start interactive command shell loop
    loop {
        // Build the dynamic colored prompt
        let prompt_str = prompt::build_prompt(last_exit_status);
        
        match editor.readline(&prompt_str) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Route command
                match commands::run_builtin(trimmed, editor.history()) {
                    commands::CommandResult::Handled(status) => {
                        last_exit_status = status;
                    }
                    commands::CommandResult::Exit => {
                        println!("\x1b[90mExiting NeoTerminal...\x1b[0m");
                        break;
                    }
                    commands::CommandResult::NotHandled => {
                        // Forward to system shell
                        last_exit_status = shell::execute_command(trimmed);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C: Print a newline and reset prompt
                println!();
                last_exit_status = 130;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D: Exit the shell
                println!("\n\x1b[90mExiting NeoTerminal (EOF)...\x1b[0m");
                break;
            }
            Err(err) => {
                eprintln!("neoterminal error: {:?}", err);
                break;
            }
        }
    }

    // 5. Save history on exit
    if let Some(ref path) = history_path {
        let _ = editor.save_history(path);
    }

    Ok(())
}

fn get_history_path() -> Option<PathBuf> {
    dirs::home_dir().map(|mut p| {
        p.push(".neoterminal_history");
        p
    })
}

fn print_greeting() {
    // Try running fastfetch
    let status_fast = std::process::Command::new("fastfetch").status();
    if status_fast.is_ok() && status_fast.unwrap().success() {
        return;
    }

    // Try running neofetch
    let status_neo = std::process::Command::new("neofetch").status();
    if status_neo.is_ok() && status_neo.unwrap().success() {
        return;
    }

    // Fallback: minimal clean prompt info
    println!("\x1b[1;36mNeoTerminal v0.1.0\x1b[0m");
    println!("Type \x1b[1;32mhelp\x1b[0m to see built-in commands.");
    println!("Press \x1b[1;31mCtrl+D\x1b[0m or type \x1b[1;31mexit\x1b[0m to exit.");
    println!();
}
