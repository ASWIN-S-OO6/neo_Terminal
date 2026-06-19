use std::fs;
use std::time::SystemTime;
use chrono::{DateTime, Local};
use crate::icons;

struct FileInfo {
    name: String,
    is_dir: bool,
    is_symlink: bool,
    is_exec: bool,
    size: u64,
    modified: SystemTime,
    permissions_str: String,
}

pub fn run(args: &[String]) -> i32 {
    let mut show_all = false;
    let mut _long_format = false;
    let mut sort_by_time = false;
    let mut sort_by_size = false;
    let mut targets = Vec::new();

    // Parse arguments
    for arg in args {
        if arg.starts_with('-') && arg.len() > 1 {
            for c in arg.chars().skip(1) {
                match c {
                    'a' => show_all = true,
                    'l' => _long_format = true,
                    't' => sort_by_time = true,
                    's' | 'S' => sort_by_size = true,
                    _ => {
                        eprintln!("ls: invalid option -- '{}'", c);
                        return 1;
                    }
                }
            }
        } else {
            targets.push(arg.clone());
        }
    }

    let target_dir = if targets.is_empty() {
        ".".to_string()
    } else {
        targets[0].clone()
    };

    let paths = match fs::read_dir(&target_dir) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("ls: cannot access '{}': {}", target_dir, e);
            return 1;
        }
    };

    let mut files = Vec::new();

    for entry_result in paths {
        if let Ok(entry) = entry_result {
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Skip hidden files if not show_all
            if !show_all && name.starts_with('.') {
                continue;
            }

            if let Ok(metadata) = entry.metadata() {
                let is_dir = metadata.is_dir();
                let is_symlink = metadata.file_type().is_symlink();
                let is_exec = is_executable(&metadata);
                let size = metadata.len();
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                let permissions_str = get_permissions_string(&metadata);

                files.push(FileInfo {
                    name,
                    is_dir,
                    is_symlink,
                    is_exec,
                    size,
                    modified,
                    permissions_str,
                });
            }
        }
    }

    // Sort files
    if sort_by_time {
        files.sort_by(|a, b| b.modified.cmp(&a.modified));
    } else if sort_by_size {
        files.sort_by(|a, b| b.size.cmp(&a.size));
    } else {
        files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }

    // Find column widths
    let mut max_name_len = 4; // "Name"
    let mut max_size_len = 4; // "Size"
    let mut max_modified_len = 8; // "Modified"
    let mut max_perms_len = 11; // "Permissions"

    struct TableRow {
        icon: String,
        colored_name: String,
        name_width: usize,
        size_str: String,
        date_str: String,
        perms_str: String,
        is_symlink: bool,
    }

    let mut rows = Vec::new();

    for file in &files {
        let icon = icons::get_icon(&file.name, file.is_dir);
        let colored_name = get_colored_name(file);
        let name_width = file.name.chars().count();
        let size_str = get_human_readable_size(file.size, file.is_dir);
        let date_str = get_modified_time_string(file.modified);
        let perms_str = file.permissions_str.clone();

        if name_width > max_name_len {
            max_name_len = name_width;
        }
        if size_str.len() > max_size_len {
            max_size_len = size_str.len();
        }
        if date_str.len() > max_modified_len {
            max_modified_len = date_str.len();
        }
        if perms_str.len() > max_perms_len {
            max_perms_len = perms_str.len();
        }

        rows.push(TableRow {
            icon,
            colored_name,
            name_width,
            size_str,
            date_str,
            perms_str,
            is_symlink: file.is_symlink,
        });
    }

    // Print headers
    println!(
        "  \x1b[90m{:2}  {:<name_w$}  {:>size_w$}  {:<mod_w$}  {:<perm_w$}\x1b[0m",
        "",
        "Name",
        "Size",
        "Modified",
        "Permissions",
        name_w = max_name_len,
        size_w = max_size_len,
        mod_w = max_modified_len,
        perm_w = max_perms_len
    );

    // Print dynamic thin divider
    println!(
        "  \x1b[90m{:2}  {:─<name_w$}  {:─>size_w$}  {:─<mod_w$}  {:─<perm_w$}\x1b[0m",
        "",
        "",
        "",
        "",
        "",
        name_w = max_name_len,
        size_w = max_size_len,
        mod_w = max_modified_len,
        perm_w = max_perms_len
    );

    // Print table rows
    for row in rows {
        // Pad the name using character count to avoid ANSI escape sequences breaking alignment
        let padding_spaces = max_name_len - row.name_width;
        let padded_name = format!("{}{}", row.colored_name, " ".repeat(padding_spaces));

        println!(
            "  {}  {}  \x1b[90m{:>size_w$}\x1b[0m  \x1b[36m{:<mod_w$}\x1b[0m  \x1b[32m{:<perm_w$}\x1b[0m{}",
            row.icon,
            padded_name,
            row.size_str,
            row.date_str,
            row.perms_str,
            if row.is_symlink { " \x1b[36m-> (symlink)\x1b[0m" } else { "" },
            size_w = max_size_len,
            mod_w = max_modified_len,
            perm_w = max_perms_len
        );
    }

    0
}

#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(_metadata: &fs::Metadata) -> bool {
    false
}

#[cfg(unix)]
fn get_permissions_string(metadata: &fs::Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    let is_dir = metadata.is_dir();
    let mut s = String::new();
    
    if is_dir {
        s.push('d');
    } else if metadata.file_type().is_symlink() {
        s.push('l');
    } else {
        s.push('-');
    }
    
    // User
    s.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o100 != 0 { 'x' } else { '-' });
    
    // Group
    s.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o010 != 0 { 'x' } else { '-' });
    
    // Others
    s.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    s.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    s.push(if mode & 0o001 != 0 { 'x' } else { '-' });
    
    s
}

#[cfg(not(unix))]
fn get_permissions_string(metadata: &fs::Metadata) -> String {
    if metadata.is_dir() {
        "d---------".to_string()
    } else {
        "----------".to_string()
    }
}

fn get_modified_time_string(modified: SystemTime) -> String {
    let dt: DateTime<Local> = modified.into();
    dt.format("%b %d %H:%M").to_string()
}

fn get_human_readable_size(size: u64, is_dir: bool) -> String {
    if is_dir {
        return "-".to_string();
    }
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

fn get_colored_name(file: &FileInfo) -> String {
    if file.is_dir {
        format!("\x1b[1;34m{}\x1b[0m", file.name) // Bold Blue
    } else if file.is_symlink {
        format!("\x1b[1;36m{}\x1b[0m", file.name) // Bold Cyan
    } else if file.is_exec {
        format!("\x1b[1;32m{}\x1b[0m", file.name) // Bold Green
    } else {
        file.name.clone() // White/Default
    }
}
