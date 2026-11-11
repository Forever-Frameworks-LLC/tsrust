use serde::{Deserialize, Serialize};

/// Shared by the CLI, language server and Studio. Missing fields use defaults.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct FormatOptions {
    pub indent_width: usize,
    pub jsx_return_parentheses: bool,
    pub jsx_children_on_new_lines: bool,
    pub blank_lines_around_blocks: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { indent_width: 2, jsx_return_parentheses: true, jsx_children_on_new_lines: true, blank_lines_around_blocks: true }
    }
}

impl FormatOptions {
    /// The nearest project overrides supplied editor defaults, field by field.
    pub fn for_path(path: &std::path::Path, base: &Self) -> Result<Self, String> {
        let start = if path.is_dir() { path } else { path.parent().unwrap_or(path) };
        for directory in start.ancestors() {
            let config = directory.join("tsrust.config.json");
            let file = match std::fs::File::open(&config) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(format!("{}: {error}", config.display())),
            };
            use std::io::Read;
            let mut text = String::new();
            file.take(1024 * 1024 + 1).read_to_string(&mut text).map_err(|error| format!("{}: {error}", config.display()))?;
            if text.len() > 1024 * 1024 { return Err("Project configuration exceeds 1 MiB.".into()); }
            let value: serde_json::Value = serde_json::from_str(&text).map_err(|error| format!("{}: {error}", config.display()))?;
            let Some(overrides) = value.get("format") else { return Ok(base.clone()); };
            return Self::overlay(base, overrides);
        }
        Ok(base.clone())
    }

    pub fn overlay(base: &Self, overrides: &serde_json::Value) -> Result<Self, String> {
        let fields = overrides.as_object().ok_or("format must be an object.")?;
        let mut value = serde_json::to_value(base).map_err(|error| error.to_string())?;
        value.as_object_mut().expect("format options object").extend(fields.clone());
        let options: Self = serde_json::from_value(value).map_err(|error| format!("Invalid format settings: {error}"))?;
        options.validate()?;
        Ok(options)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(1..=8).contains(&self.indent_width) {
            return Err("format.indentWidth must be between 1 and 8.".into());
        }
        Ok(())
    }
}
