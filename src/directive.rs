use anyhow::Context;
use regex::Regex;

pub(crate) struct Directive {
    pub(crate) file_path: String,
    pub(crate) item: Option<String>,
    pub(crate) extra_items: Vec<String>,
}

pub(crate) fn parse_directive_args(directive: &str) -> anyhow::Result<Directive> {
    // Basic regex to parse directive: directive_name!("path/to/file.rs", item_name, [deps...])
    let re = Regex::new(
        r#"([a-z_]+)!\s*\(\s*"([^"]+)"\s*(?:,\s*([^\[\]]+?))?\s*(?:,\s*\[(.*)\])?\s*\)"#,
    )?;

    let captures = re
        .captures(directive)
        .with_context(|| format!("Failed to parse directive: {}", directive))?;

    let file_path = captures
        .get(2)
        .map(|m| m.as_str().to_string())
        .with_context(|| "File path is required")?;

    let item = captures.get(3).map(|m| m.as_str().trim().to_string());

    let dependencies = captures
        .get(4)
        .map(|m| {
            m.as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    Ok(Directive {
        file_path,
        item,
        extra_items: dependencies,
    })
}
