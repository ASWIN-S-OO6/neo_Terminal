use walkdir::WalkDir;
use crate::icons;

pub fn run(pattern: &str) -> i32 {
    let lowercase_pattern = pattern.to_lowercase();
    let mut count = 0;

    println!("\x1b[90mSearching recursively for '{}'...\x1b[0m", pattern);

    for entry_result in WalkDir::new(".") {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let name = entry.file_name().to_string_lossy().to_string();
        if name.to_lowercase().contains(&lowercase_pattern) {
            let path = entry.path();
            let is_dir = entry.file_type().is_dir();
            
            // Format relative path for output
            let path_str = if let Ok(stripped) = path.strip_prefix(".") {
                stripped.to_string_lossy().to_string()
            } else {
                path.to_string_lossy().to_string()
            };

            // Don't print empty path or just dot
            if path_str.is_empty() {
                continue;
            }

            let icon = icons::get_icon(&name, is_dir);
            let colored_name = if is_dir {
                format!("\x1b[1;34m{}\x1b[0m", name) // Bold Blue
            } else if entry.file_type().is_symlink() {
                format!("\x1b[1;36m{}\x1b[0m", name) // Bold Cyan
            } else {
                name.clone() // White/Default
            };

            println!("  {}{}  \x1b[90m({})\x1b[0m", icon, colored_name, path_str);
            count += 1;
        }
    }

    println!("\nFound \x1b[1;36m{}\x1b[0m match{}", count, if count == 1 { "" } else { "es" });
    0
}
