use std::process::Command;

/// Run a command string via the system shell (`sh -c`)
/// and return the exit code of the process.
pub fn execute_command(line: &str) -> i32 {
    let mut child = match Command::new("sh")
        .arg("-c")
        .arg(line)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("neoterminal: failed to start process: {}", e);
            return 127;
        }
    };

    // Wait for the command to finish and get exit code
    match child.wait() {
        Ok(status) => {
            // Unix process exit code, fallback to 0 if terminated by signal
            status.code().unwrap_or(0)
        }
        Err(e) => {
            eprintln!("neoterminal: error while waiting for process: {}", e);
            1
        }
    }
}
