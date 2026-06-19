use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::icons::IconMode;

/// Fetch the git branch name and dirty status of the current working directory
fn get_git_info() -> Option<(String, bool)> {
    let is_git = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();

    match is_git {
        Ok(out) if out.status.success() => {}
        _ => return None,
    };

    // Get current branch name
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .ok()?;
    
    let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
    if branch.is_empty() {
        return None;
    }

    // Get dirty status
    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    
    let is_dirty = !status_output.stdout.is_empty();

    Some((branch, is_dirty))
}

/// Get the system hostname
fn get_hostname() -> String {
    if let Ok(name) = fs::read_to_string("/proc/sys/kernel/hostname") {
        let trimmed = name.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    env::var("HOSTNAME")
        .or_else(|_| env::var("HOST"))
        .unwrap_or_else(|_| "host".to_string())
}

/// Shorten path for display
fn get_shortened_path() -> String {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let home_dir = dirs::home_dir();

    let path_str = if let Some(ref home) = home_dir {
        if let Ok(stripped) = current_dir.strip_prefix(home) {
            let relative = stripped.to_string_lossy();
            if relative.is_empty() {
                "~".to_string()
            } else {
                format!("~/{}", relative)
            }
        } else {
            current_dir.to_string_lossy().to_string()
        }
    } else {
        current_dir.to_string_lossy().to_string()
    };

    // If path is too long (contains more than 3 directory levels), shorten the middle
    let parts: Vec<&str> = path_str.split('/').collect();
    if parts.len() > 4 {
        let first = parts[0];
        let last_three = &parts[parts.len() - 3..];
        if first == "~" {
            format!("~/.../{}", last_three.join("/"))
        } else {
            format!("{}/.../{}", first, last_three.join("/"))
        }
    } else {
        path_str
    }
}

/// Generates the dynamic terminal prompt string using a sleek, professional Kali-inspired bracket layout
pub fn build_prompt(last_exit_status: i32) -> String {
    let mode = IconMode::current();

    // 1. Bracket color: Deep Kali Blue (rgb(0, 136, 204))
    let bracket_color = "\x1b[38;2;0;136;204m";
    let reset = "\x1b[0m";

    // 2. Identity: (user㉿host)
    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());
    let hostname = get_hostname();
    
    let identity_color = "\x1b[38;2;26;188;156m"; // Mint/Teal
    let sep_symbol = "@";
    let sep_color = "\x1b[38;2;120;120;120m"; // Muted Gray
    let identity_str = format!(
        "{identity_color}{username}{reset}{sep_color}{sep_symbol}{reset}{identity_color}{hostname}{reset}"
    );

    // 3. Current Directory Path: [ ~/.../path]
    let path_str = get_shortened_path();
    let path_display = match mode {
        IconMode::Nerd => format!("\u{f07b} {}", path_str),
        IconMode::Emoji => format!("📁 {}", path_str),
    };
    let path_str_colored = format!("\x1b[1;37m{}\x1b[0m", path_display); // Bright White path

    // 4. Git Branch: [ branch*]
    let git_str = if let Some((branch, is_dirty)) = get_git_info() {
        let dirty_symbol = if is_dirty { "*" } else { "" };
        let git_icon = match mode {
            IconMode::Nerd => "\u{e725} ",
            IconMode::Emoji => "🌿 ",
        };
        format!(
            "─{bracket_color}[{reset}\x1b[38;2;230;126;34m{}{}{}{reset}{bracket_color}]{reset}",
            git_icon, branch, dirty_symbol
        )
    } else {
        "".to_string()
    };

    // 5. Exit Status (only show if failure)
    let status_str = if last_exit_status != 0 {
        format!(
            "─{bracket_color}[{reset}\x1b[38;2;231;76;60m✘ {last_exit_status}{reset}{bracket_color}]{reset}"
        )
    } else {
        "".to_string()
    };

    // Build line 1: ┌──(user㉿host)─[path]─[git]─[status]
    let line1 = format!(
        "{bracket_color}┌──{reset}({identity})─{bracket_color}[{reset}{path}{bracket_color}]{reset}{git}{status}",
        bracket_color = bracket_color,
        reset = reset,
        identity = identity_str,
        path = path_str_colored,
        git = git_str,
        status = status_str
    );

    // Build line 2: └─❯ 
    let arrow_color = if last_exit_status == 0 {
        "\x1b[1;32m" // Green on success
    } else {
        "\x1b[1;31m" // Red on failure
    };

    format!(
        "\n{}\n{bracket_color}└─{reset}{arrow_color}❯{reset} ",
        line1
    )
}
