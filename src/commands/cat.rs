use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};
use std::fs;
use std::path::Path;

pub fn run(file_path: &str) -> i32 {
    let path = Path::new(file_path);
    if !path.exists() {
        eprintln!("cat: {}: No such file or directory", file_path);
        return 1;
    }
    if path.is_dir() {
        eprintln!("cat: {}: Is a directory", file_path);
        return 1;
    }

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("cat: {}: {}", file_path, e);
            return 1;
        }
    };

    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Determine syntax by extension, falling back to plain text
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let syntax = ps.find_syntax_by_extension(ext)
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    let theme = &ts.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);

    let mut line_num = 1;
    for line in LinesWithEndings::from(&content) {
        let ranges: Vec<(Style, &str)> = match h.highlight_line(line, &ps) {
            Ok(r) => r,
            Err(_) => {
                // Fallback if highlighting fails
                print!("\x1b[90m{:4} |\x1b[0m {}", line_num, line);
                line_num += 1;
                continue;
            }
        };
        // Print 24-bit terminal escaped color code
        let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
        
        print!("\x1b[90m{:4} |\x1b[0m {}", line_num, escaped);
        line_num += 1;
    }

    // Reset terminal style
    print!("\x1b[0m");
    if !content.ends_with('\n') {
        println!();
    }

    0
}
