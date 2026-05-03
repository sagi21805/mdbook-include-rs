pub(crate) mod const_finder;
pub(crate) mod enum_finder;
pub(crate) mod function_extractor;
pub(crate) mod impl_finder;
pub(crate) mod static_finder;
pub(crate) mod struct_finder;
pub(crate) mod trait_finder;

use crate::parser::get_relative_path;
use crate::permissive_file::parse_file_permissive;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use syn::File;

/// Read and parse a Rust source file
pub(crate) fn read_and_parse_file(file_path: &Path) -> Result<File> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", get_relative_path(file_path)))?;

    // Pretty print the code for consistent formatting
    let syntax_tree = parse_file_permissive(&content)
        .with_context(|| format!("Failed to parse file: {}", get_relative_path(file_path)))?;
    Ok(syntax_tree)
}
