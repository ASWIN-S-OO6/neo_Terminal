use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{MatchingBracketValidator, Validator, ValidationContext, ValidationResult};
use rustyline::{Context, Helper, Result};
use std::borrow::Cow;
use std::collections::HashSet;
use std::env;
use std::fs;

const BUILTINS: &[&str] = &[
    "ls", "cd", "cat", "view", "sys", "find", "history", "help", "exit", "clear"
];

pub struct NeoHelper {
    pub completer: FilenameCompleter,
    pub validator: MatchingBracketValidator,
    pub hinter: HistoryHinter,
    pub system_commands: HashSet<String>,
}

impl NeoHelper {
    pub fn new() -> Self {
        Self {
            completer: FilenameCompleter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter::new(),
            system_commands: get_system_commands(),
        }
    }
}

impl Helper for NeoHelper {}

impl Completer for NeoHelper {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        let slice = &line[..pos];
        
        // If there are no spaces in the current input up to the cursor, we autocomplete commands
        if !slice.contains(' ') {
            let mut matches = Vec::new();
            
            // Add builtins that start with the input slice
            for cmd in BUILTINS {
                if cmd.starts_with(slice) {
                    matches.push(Pair {
                        display: cmd.to_string(),
                        replacement: format!("{} ", cmd),
                    });
                }
            }
            
            // Add system commands that start with the input slice
            let mut sys_cmds: Vec<&String> = self.system_commands
                .iter()
                .filter(|cmd| cmd.starts_with(slice))
                .collect();
            sys_cmds.sort();
            
            for cmd in sys_cmds {
                if !BUILTINS.contains(&cmd.as_str()) {
                    matches.push(Pair {
                        display: cmd.to_string(),
                        replacement: format!("{} ", cmd),
                    });
                }
            }

            if !matches.is_empty() {
                return Ok((0, matches));
            }
        }

        // Otherwise, delegate to the standard filename completer
        self.completer.complete(line, pos, ctx)
    }
}

impl Highlighter for NeoHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if line.is_empty() {
            return Cow::Borrowed(line);
        }

        let mut highlighted = String::new();
        let mut chars = line.chars().peekable();
        let mut current_token = String::new();
        let mut is_first_token = true;
        let mut in_quote = None;

        let flush_token = |token: &mut String, hl: &mut String, is_first: &mut bool, in_q: Option<char>| {
            if token.is_empty() {
                return;
            }

            // Strip quotes to check path existence
            let clean_token = if (token.starts_with('"') && token.ends_with('"')) 
                || (token.starts_with('\'') && token.ends_with('\'')) {
                if token.len() >= 2 {
                    &token[1..token.len()-1]
                } else {
                    token.as_str()
                }
            } else {
                token.as_str()
            };

            let path_exists = !clean_token.is_empty() && std::path::Path::new(clean_token).exists();

            if *is_first {
                let is_valid = BUILTINS.contains(&token.as_str())
                    || self.system_commands.contains(token.as_str())
                    || (token.starts_with('.') || token.starts_with('/')) && std::path::Path::new(token).exists();

                if is_valid {
                    // Valid Command: Bold Darker Blue (rgb(41, 128, 185) - no purple)
                    hl.push_str(&format!("\x1b[1;38;2;41;128;185m{}\x1b[0m", token));
                } else {
                    // Invalid Command: Bold Red
                    hl.push_str(&format!("\x1b[1;31m{}\x1b[0m", token));
                }
                *is_first = false;
            } else if path_exists {
                // Existing File/Folder/Path: Bright White (like Konsole/Kali)
                hl.push_str(&format!("\x1b[1;37m{}\x1b[0m", token));
            } else if in_q.is_some() {
                // String: Warm Coral/Orange (rgb(230, 126, 34))
                hl.push_str(&format!("\x1b[38;2;230;126;34m{}\x1b[0m", token));
            } else if token.starts_with('-') {
                // Option/Flag: Mint/Teal (rgb(26, 188, 156))
                hl.push_str(&format!("\x1b[38;2;26;188;156m{}\x1b[0m", token));
            } else {
                // Regular argument: Soft Gray (rgb(200, 200, 200))
                hl.push_str(&format!("\x1b[38;2;200;200;200m{}\x1b[0m", token));
            }
            token.clear();
        };

        while let Some(c) = chars.next() {
            if let Some(q) = in_quote {
                current_token.push(c);
                if c == q {
                    flush_token(&mut current_token, &mut highlighted, &mut is_first_token, Some(q));
                    in_quote = None;
                }
            } else {
                match c {
                    '"' | '\'' => {
                        flush_token(&mut current_token, &mut highlighted, &mut is_first_token, None);
                        in_quote = Some(c);
                        current_token.push(c);
                    }
                    ' ' | '\t' => {
                        flush_token(&mut current_token, &mut highlighted, &mut is_first_token, None);
                        highlighted.push(c);
                    }
                    _ => {
                        current_token.push(c);
                    }
                }
            }
        }

        flush_token(&mut current_token, &mut highlighted, &mut is_first_token, in_quote);

        Cow::Owned(highlighted)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: rustyline::highlight::CmdKind) -> bool {
        true
    }
}

impl Hinter for NeoHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Validator for NeoHelper {
    fn validate(&self, ctx: &mut ValidationContext<'_>) -> Result<ValidationResult> {
        self.validator.validate(ctx)
    }
}

/// Retrieve all executable command names in system's PATH
fn get_system_commands() -> HashSet<String> {
    let mut commands = HashSet::new();
    if let Ok(path_var) = env::var("PATH") {
        for path in env::split_paths(&path_var) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Ok(meta) = entry.metadata() {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if meta.is_file() && (meta.permissions().mode() & 0o111 != 0) {
                                if let Some(name) = entry.file_name().to_str() {
                                    commands.insert(name.to_string());
                                }
                            }
                        }
                        #[cfg(not(unix))]
                        {
                            if meta.is_file() {
                                if let Some(name) = entry.file_name().to_str() {
                                    commands.insert(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    commands
}
