use std::env;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconMode {
    Emoji,
    Nerd,
}

impl IconMode {
    pub fn current() -> Self {
        match env::var("NEOTERMINAL_ICONS").as_deref() {
            Ok("emoji") => IconMode::Emoji,
            _ => IconMode::Nerd,
        }
    }
}

pub fn get_icon(name: &str, is_dir: bool) -> String {
    let mode = IconMode::current();
    if is_dir {
        return match mode {
            // Folder icon: \u{f07b} ( in Nerd Fonts/Font Awesome)
            IconMode::Nerd => "\u{f07b} ".to_string(),
            IconMode::Emoji => "📁 ".to_string(),
        };
    }

    match mode {
        IconMode::Nerd => {
            let path = Path::new(name);
            let file_icon = devicons::FileIcon::from(path);
            format!("{} ", file_icon.icon)
        }
        IconMode::Emoji => {
            let lowercase_name = name.to_lowercase();
            let parts: Vec<&str> = lowercase_name.split('.').collect();
            let ext = parts.last().copied().unwrap_or("");

            let emoji = match lowercase_name.as_str() {
                "cargo.toml" => "🦀 ",
                "cargo.lock" => "🔒 ",
                "makefile" => "🛠️ ",
                "dockerfile" | "docker-compose.yml" | "docker-compose.yaml" => "🐳 ",
                "gitignore" | ".gitignore" => "🌿 ",
                "license" | "license.txt" | "license.md" | "copying" => "⚖️ ",
                _ => match ext {
                    "rs" => "🦀 ",
                    "py" => "🐍 ",
                    "js" => "🟨 ",
                    "ts" => "🟦 ",
                    "jsx" | "tsx" => "⚛️ ",
                    "html" | "htm" => "🌐 ",
                    "css" | "scss" | "sass" | "less" => "🎨 ",
                    "json" | "toml" | "yaml" | "yml" => "⚙️ ",
                    "md" => "📘 ",
                    "sh" | "bash" | "zsh" | "fish" | "bat" | "cmd" | "ps1" => "⚡ ",
                    "go" => "🐹 ",
                    "cpp" | "cc" | "cxx" | "h" | "hpp" | "c" => "🔹 ",
                    "java" | "class" => "☕ ",
                    "php" => "🐘 ",
                    "rb" => "💎 ",
                    "sql" | "db" | "sqlite" | "sqlite3" => "🗄️ ",
                    "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "bmp" | "ico" => "🖼️ ",
                    "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" => "🎵 ",
                    "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" => "🎥 ",
                    "zip" | "tar" | "gz" | "tgz" | "rar" | "7z" | "bz2" | "xz" => "📦 ",
                    "pdf" => "📕 ",
                    "txt" | "log" | "rtf" | "doc" | "docx" | "odt" => "📄 ",
                    "xls" | "xlsx" | "ods" | "csv" => "📊 ",
                    "ppt" | "pptx" | "odp" => "📈 ",
                    _ => "📄 ",
                }
            };
            emoji.to_string()
        }
    }
}
