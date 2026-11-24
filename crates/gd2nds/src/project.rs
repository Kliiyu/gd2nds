//! Minimal reader for Godot 4 `project.godot` files.
//!
//! Godot's text config format is INI-like: `[section]` headers, `key=value`
//! lines and `;` comments. Values use Godot's constructor syntax. Milestone 1
//! only needs a handful of plain string and integer values, so this reader keeps
//! values as raw text and offers helpers to decode simple strings.

use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

/// The Godot minor version gd2nds targets.
pub const TARGET_GODOT_VERSION: &str = "4.7";

/// `config_version` written by Godot 4.x.
const GODOT4_CONFIG_VERSION: &str = "5";

#[derive(Debug)]
pub enum ProjectError {
    Io(std::io::Error),
    Parse { line: usize, message: String },
    NotGodot4 { found: Option<String> },
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectError::Io(e) => write!(f, "could not read project.godot: {e}"),
            ProjectError::Parse { line, message } => {
                write!(f, "project.godot:{line}: {message}")
            }
            ProjectError::NotGodot4 { found } => match found {
                Some(v) => write!(
                    f,
                    "project.godot has config_version={v}; gd2nds needs a Godot 4 project (config_version=5)"
                ),
                None => write!(
                    f,
                    "project.godot has no config_version; gd2nds needs a Godot 4 project"
                ),
            },
        }
    }
}

impl std::error::Error for ProjectError {}

impl From<std::io::Error> for ProjectError {
    fn from(e: std::io::Error) -> Self {
        ProjectError::Io(e)
    }
}

/// Raw key/value pairs of a `project.godot`, grouped by section.
/// Keys before the first section live in the "" section.
#[derive(Debug, Default)]
pub struct ProjectConfig {
    sections: BTreeMap<String, BTreeMap<String, String>>,
}

impl ProjectConfig {
    pub fn load(project_dir: &Path) -> Result<Self, ProjectError> {
        let text = std::fs::read_to_string(project_dir.join("project.godot"))?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Self, ProjectError> {
        let mut config = ProjectConfig::default();
        let mut section = String::new();
        let mut lines = text.lines().enumerate().peekable();

        while let Some((idx, raw)) = lines.next() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if let Some(rest) = line.strip_prefix('[') {
                let name = rest.strip_suffix(']').ok_or(ProjectError::Parse {
                    line: idx + 1,
                    message: "section header is missing ']'".into(),
                })?;
                section = name.trim().to_string();
                continue;
            }
            let (key, value) = line.split_once('=').ok_or(ProjectError::Parse {
                line: idx + 1,
                message: format!("expected key=value, found {line:?}"),
            })?;

            // Values such as dictionaries and arrays can span several lines.
            // Keep reading until brackets and quotes balance.
            let mut value = value.trim().to_string();
            while !is_balanced(&value) {
                match lines.next() {
                    Some((_, more)) => {
                        value.push('\n');
                        value.push_str(more);
                    }
                    None => {
                        return Err(ProjectError::Parse {
                            line: idx + 1,
                            message: format!("unterminated value for key {:?}", key.trim()),
                        })
                    }
                }
            }

            config
                .sections
                .entry(section.clone())
                .or_default()
                .insert(key.trim().to_string(), value);
        }

        let version = config.get("", "config_version").map(str::to_string);
        if version.as_deref() != Some(GODOT4_CONFIG_VERSION) {
            return Err(ProjectError::NotGodot4 { found: version });
        }
        Ok(config)
    }

    /// The raw text of a value, exactly as written after `=`.
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections.get(section)?.get(key).map(String::as_str)
    }

    /// A value decoded as a Godot string literal (`"..."`).
    pub fn get_string(&self, section: &str, key: &str) -> Option<String> {
        decode_string(self.get(section, key)?)
    }

    pub fn name(&self) -> String {
        self.get_string("application", "config/name")
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| "Untitled".to_string())
    }

    /// Feature strings from `config/features`, such as `["4.7", "GL Compatibility"]`.
    pub fn features(&self) -> Vec<String> {
        let Some(raw) = self.get("application", "config/features") else {
            return Vec::new();
        };
        let inner = raw
            .trim()
            .strip_prefix("PackedStringArray(")
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or("");
        split_string_list(inner)
    }

    /// The Godot minor version the project was saved with, if recorded.
    pub fn godot_version(&self) -> Option<String> {
        self.features()
            .into_iter()
            .find(|f| f.starts_with(|c: char| c.is_ascii_digit()) && f.contains('.'))
    }
}

fn is_balanced(value: &str) -> bool {
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    for c in value.chars() {
        if in_string {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            _ => {}
        }
    }
    depth <= 0 && !in_string
}

/// Decode a Godot string literal. Returns `None` if `raw` is not a string.
pub fn decode_string(raw: &str) -> Option<String> {
    let inner = raw.trim().strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    Some(out)
}

fn split_string_list(inner: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for c in inner.chars() {
        if in_string {
            current.push(c);
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
        } else if c == '"' {
            in_string = true;
            current.push(c);
        } else if c == ',' {
            items.extend(decode_string(&current));
            current.clear();
        } else {
            current.push(c);
        }
    }
    items.extend(decode_string(&current));
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"; Engine configuration file.
config_version=5

[application]

config/name="Hello \"DS\""
config/features=PackedStringArray("4.7", "GL Compatibility")

[input]

jump={
"deadzone": 0.5,
"events": []
}
"#;

    #[test]
    fn reads_name_and_version() {
        let cfg = ProjectConfig::parse(SAMPLE).unwrap();
        assert_eq!(cfg.name(), "Hello \"DS\"");
        assert_eq!(cfg.godot_version().as_deref(), Some("4.7"));
        assert_eq!(cfg.features(), vec!["4.7", "GL Compatibility"]);
    }

    #[test]
    fn reads_multiline_values() {
        let cfg = ProjectConfig::parse(SAMPLE).unwrap();
        let jump = cfg.get("input", "jump").unwrap();
        assert!(jump.starts_with('{') && jump.ends_with('}'));
    }

    #[test]
    fn rejects_godot3_projects() {
        let err = ProjectConfig::parse("config_version=4\n").unwrap_err();
        assert!(matches!(err, ProjectError::NotGodot4 { .. }));
    }

    #[test]
    fn missing_name_falls_back() {
        let cfg = ProjectConfig::parse("config_version=5\n").unwrap();
        assert_eq!(cfg.name(), "Untitled");
    }
}
