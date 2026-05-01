use crate::directive::parse_directive_args;
use crate::extractor::const_finder::find_const;
use crate::extractor::enum_finder::find_enum;
use crate::extractor::function_extractor::find_function;
use crate::extractor::impl_finder::{find_impl_methods, find_struct_impl, find_trait_impl};
use crate::extractor::read_and_parse_file;
use crate::extractor::static_finder::find_static;
use crate::extractor::struct_finder::find_struct;
use crate::extractor::trait_finder::find_trait;
use crate::formatter::{format_function_body, format_item};
use crate::output::Output;
use anyhow::{Context, Result};
use proc_macro2::Span;
use regex::{Captures, Regex};
use std::path::{Path, PathBuf};
use std::{env, fs};
use syn::spanned::Spanned;
use syn::token::{Enum, Impl, Struct, Trait};
use syn::{File, Item, ItemConst, ItemFn, ItemStatic};

const DIRECTIVE_REGEX: &str = r"(?ms)^#!\[((?:source_file|static|const|function|struct|enum|trait|impl|impl_method|trait_impl|function_body)![\s\S]*?)\]$";

/// Process the markdown content to find and replace include-rs directives
pub fn process_markdown(base_dir: &Path, source_path: &Path, content: &mut String) -> Result<()> {
    // This regex finds our directives anywhere in the content
    let re = Regex::new(DIRECTIVE_REGEX)?;

    // Track the start position of each line to calculate line numbers
    let mut line_positions = Vec::new();
    let mut pos = 0;
    for line in content.lines() {
        line_positions.push(pos);
        pos += line.len() + 1; // +1 for the newline character
    }

    let result = re.replace_all(content, |caps: &Captures| {
        let include_doc_directive = caps.get(1).map_or("", |m| m.as_str());

        // Get match position information
        let match_start = caps.get(0).map_or(0, |m| m.start());

        // Find line number and column based on position
        let (line_num, col_num) = find_line_and_col(&line_positions, match_start);

        // Process the directive with include_doc_macro
        match process_include_rs_directive(base_dir, include_doc_directive) {
            Ok((processed, _, _)) => processed,
            Err(e) => {
                let rel_path = get_relative_path(source_path);
                eprintln!("{}:{}:{}: {}", rel_path, line_num, col_num, e);
                format!("{}:{}:{}: {}", rel_path, line_num, col_num, e)
            }
        }
    });

    *content = result.to_string();
    Ok(())
}

pub fn process_directives(
    base_dir: &Path,
    source_path: &Path,
    content: &str,
) -> Result<Vec<(PathBuf, Vec<Span>)>> {
    // Changed return type to Vec<Span>
    let re = Regex::new(DIRECTIVE_REGEX)?;

    let mut line_positions = Vec::new();
    let mut pos = 0;
    for line in content.lines() {
        line_positions.push(pos);
        pos += line.len() + 1;
    }

    let mut spans_by_path = Vec::new();

    for caps in re.captures_iter(content) {
        let include_doc_directive = caps.get(1).map_or("", |m| m.as_str());
        let match_start = caps.get(0).map_or(0, |m| m.start());
        let (line_num, col_num) = find_line_and_col(&line_positions, match_start);

        match process_include_rs_directive(base_dir, include_doc_directive) {
            Ok((_, path, spans)) => {
                if let Some(path) = path {
                    spans_by_path.push((path, spans));
                }
            }
            Err(e) => {
                let rel_path = get_relative_path(source_path);
                eprintln!("{}:{}:{}: {}", rel_path, line_num, col_num, e);
                continue;
            }
        }
    }

    Ok(spans_by_path)
}

/// Find line and column number from a position in the text
fn find_line_and_col(line_positions: &[usize], position: usize) -> (usize, usize) {
    let mut line_idx = 0;

    // Find the line containing the position
    for (idx, &start) in line_positions.iter().enumerate() {
        if position >= start {
            line_idx = idx;
        } else {
            break;
        }
    }

    // Line numbers are 1-indexed
    let line_num = line_idx + 1;
    // Calculate column number (1-indexed)
    let col_num = position - line_positions[line_idx] + 1;

    (line_num, col_num)
}

/// Get the path relative to the current working directory
pub(crate) fn get_relative_path(path: &Path) -> String {
    if let Ok(current_dir) = env::current_dir() {
        if let Ok(relative) = path.strip_prefix(&current_dir) {
            return format!(
                ".{}{}",
                std::path::MAIN_SEPARATOR,
                relative.to_string_lossy()
            );
        }
    }

    // Fall back to the original path if we can't get a relative path
    format!(".{}{}", std::path::MAIN_SEPARATOR, path.to_string_lossy())
}

/// Process an include-rs directive
fn process_include_rs_directive(
    base_dir: &Path,
    directive: &str,
) -> Result<(String, Option<PathBuf>, Vec<Span>)> {
    // Returns Option<PathBuf>
    let directive_name = if let Some(pos) = directive.find('!') {
        &directive[0..pos]
    } else {
        return Ok((directive.to_string(), None, Vec::new()));
    };

    let (result, path, spans) = match directive_name {
        "source_file" => {
            let (content, path) = process_source_file_directive(base_dir, directive)?;
            (content, Some(path), Vec::<Span>::new()) // Added explicit type hint
        }

        "const" => process_directive::<ItemConst>(
            base_dir,
            directive,
            |f, n| find_const(f, n).map(|item| (Item::Const(item), Vec::new())),
            format_item,
        )
        .map(|(s, p, v)| (s, Some(p), v))?,

        "static" => process_directive::<ItemStatic>(
            base_dir,
            directive,
            |f, n| find_static(f, n).map(|item| (Item::Static(item), Vec::new())),
            format_item,
        )
        .map(|(s, p, v)| (s, Some(p), v))?,

        "function_body" => process_directive::<ItemFn>(
            base_dir,
            directive,
            |f, n| find_function(f, n).map(|item| (Item::Fn(item), Vec::new())),
            format_function_body,
        )
        .map(|(s, p, v)| (s, Some(p), v))?, // Wrap path in Some

        "struct" => process_directive::<Struct>(
            base_dir,
            directive,
            |f, n| find_struct(f, n).map(|item| (Item::Struct(item), Vec::new())),
            format_item,
        )
        .map(|(s, p, v)| (s, Some(p), v))?,

        "enum" => process_directive::<Enum>(
            base_dir,
            directive,
            |f, n| find_enum(f, n).map(|item| (Item::Enum(item), Vec::new())),
            format_item,
        )
        .map(|(s, p, v)| (s, Some(p), v))?,

        "trait" => process_directive::<Trait>(
            base_dir,
            directive,
            |f, n| find_trait(f, n).map(|item| (Item::Trait(item), Vec::new())),
            format_item,
        )
        .map(|(s, p, v)| (s, Some(p), v))?,

        "impl" => process_directive::<Impl>(
            base_dir,
            directive,
            |f, n| find_struct_impl(f, n).map(|item| (item.item(), Vec::new())),
            format_item,
        )
        .map(|(s, p, v)| (s, Some(p), v))?,

        "trait_impl" => process_directive::<Impl>(
            base_dir,
            directive,
            |f, n| {
                let parts: Vec<&str> = n.split(" for ").collect();
                if parts.len() != 2 {
                    return None;
                }
                let trait_name = parts[0].trim();
                let struct_name = parts[1].trim();
                let (items, spans) = find_trait_impl(f, trait_name, struct_name);
                // TODO: good enough for book, but won't include all the currect string. The correct fix is to only use spans, and then at the end of processing include them directly from the file.
                Some((items[0].clone().item(), spans))
            },
            format_item,
        )
        .map(|(s, p, v)| (s, Some(p), v))?,

        "function" => process_directive::<ItemFn>(
            base_dir,
            directive,
            |f, n| find_function(f, n).map(|item| (Item::Fn(item), Vec::new())),
            format_item,
        )
        .map(|(s, p, v)| (s, Some(p), v))?,

        "impl_method" => process_directive::<Impl>(
            base_dir,
            directive,
            |f, n| {
                let (struct_name, methods_raw) = n.split_once("::")?;
                let struct_name = struct_name.trim();

                let method_names: Vec<&str> = methods_raw
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();

                let item_impl = find_struct_impl(f, struct_name)?;

                let (_, spans) = find_impl_methods(f, struct_name, method_names);

                let new_impl = item_impl.clone();
                // TODO: good enough for book, but won't include all the currect string. The correct fix is to only use spans, and then at the end of processing include them directly from the file.
                Some((new_impl.item(), spans))
            },
            format_item,
        )
        .map(|(s, p, v)| (s, Some(p), v))?,
        _ => return Ok((directive.to_string(), None, Vec::new())),
    };

    Ok((result.trim().to_string(), path, spans))
}

/// Process source_file! directive
fn process_source_file_directive(base_dir: &Path, directive: &str) -> Result<(String, PathBuf)> {
    let directive = parse_directive_args(directive)?;
    let absolute_path = base_dir.join(directive.file_path);
    let content = fs::read_to_string(&absolute_path)
        .with_context(|| format!("Failed to read file: {}", get_relative_path(&absolute_path)))?;
    Ok((content, absolute_path))
}

/// Helper function to process extra items
fn process_extra(
    parsed_file: &File,
    primary_item: &Item,
    extra_items: &[String],
) -> (Vec<Item>, Vec<Item>) {
    let mut hidden = Vec::new();
    let mut visible = Vec::new();

    for item in extra_items {
        if item.starts_with("struct ") {
            let struct_name = item.trim_start_matches("struct ").trim();
            if let Some(struct_def) = find_struct(parsed_file, struct_name) {
                visible.push(Item::Struct(struct_def));
            }
        } else if item.starts_with("enum ") {
            let enum_name = item.trim_start_matches("enum ").trim();
            if let Some(enum_def) = find_enum(parsed_file, enum_name) {
                visible.push(Item::Enum(enum_def));
            }
        } else if item.starts_with("trait ") {
            let trait_name = item.trim_start_matches("trait ").trim();
            if let Some(trait_def) = find_trait(parsed_file, trait_name) {
                visible.push(Item::Trait(trait_def));
            }
        } else if item.starts_with("impl ") {
            if item.contains(" for ") {
                // Trait implementation for a struct
                let parts: Vec<&str> = item.trim_start_matches("impl ").split(" for ").collect();
                if parts.len() == 2 {
                    let trait_name = parts[0].trim();
                    let struct_name = parts[1].trim();

                    for impl_def in find_trait_impl(parsed_file, trait_name, struct_name).0 {
                        visible.push(impl_def.item());
                    }
                }
            } else {
                // Struct implementation
                let struct_name = item.trim_start_matches("impl ").trim();
                if let Some(impl_def) = find_struct_impl(parsed_file, struct_name) {
                    visible.push(impl_def.item());
                }
            }
        } else {
            // Assume it's a struct or enum
            if let Some(struct_def) = find_struct(parsed_file, item) {
                visible.push(Item::Struct(struct_def));
            } else if let Some(enum_def) = find_enum(parsed_file, item) {
                visible.push(Item::Enum(enum_def));
            }
        }
    }

    // Now go through every item in the file, and if it's not in visible it must be hidden
    for item in &parsed_file.items {
        if item == primary_item {
            continue;
        }
        if !visible.contains(item) {
            hidden.push(item.clone());
        }
    }

    (hidden, visible)
}

/// Process enum! directive
fn process_directive<T>(
    base_dir: &Path,
    directive: &str,
    finder: impl Fn(&File, &str) -> Option<(Item, Vec<Span>)>,
    formatter: impl Fn(&Item) -> String,
) -> Result<(String, PathBuf, Vec<Span>)> {
    let directive = parse_directive_args(directive)?;
    let item_name = directive
        .item
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("{} name is required", std::any::type_name::<T>()))?;

    let absolute_path = base_dir.join(directive.file_path);
    let parsed_file = read_and_parse_file(&absolute_path)?;

    let (item, mut result_spans) = finder(&parsed_file, item_name)
        .with_context(|| format!("{} '{}' not found", std::any::type_name::<T>(), item_name))?;

    // If the finder didn't provide specific spans, use the item's own span
    if result_spans.is_empty() {
        result_spans.push(item.span());
    }

    let (hidden_deps, visible_deps) = process_extra(&parsed_file, &item, &directive.extra_items);
    let mut result = Output::new();

    for dep in hidden_deps {
        result.add_hidden_content(format_item(&dep));
    }
    for dep in visible_deps {
        result.add_visible_content(format_item(&dep));
    }

    result.add_visible_content(formatter(&item));

    Ok((result.format(), absolute_path, result_spans))
}
