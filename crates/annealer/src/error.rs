use thiserror::Error;

/// Errors raised while formatting a document.
#[derive(Debug, Error)]
pub enum FormatError {
    #[error("unterminated start tag at {line}:{column}")]
    UnterminatedTag { line: usize, column: usize },

    #[error("unterminated attribute value at {line}:{column}")]
    UnterminatedAttributeValue { line: usize, column: usize },

    #[error("failed to format stylesheet starting at {line}:{column}: {message}")]
    Stylesheet {
        line: usize,
        column: usize,
        message: String,
    },
}

/// Errors raised while loading or validating a profile.
#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("invalid profile YAML: {0}")]
    Yaml(#[from] yaml_serde::Error),

    #[error("unsupported schemaVersion {0} (supported: 1)")]
    SchemaVersion(u32),

    #[error("a profile must declare exactly one fallback group, found {0}")]
    Fallback(usize),

    #[error("profile and group names must not be empty")]
    EmptyName,

    #[error("duplicate group name `{0}`")]
    DuplicateGroup(String),

    #[error("group `{0}` has no `match` patterns and is not the fallback group")]
    EmptyGroup(String),

    #[error(
        "invalid pattern `{pattern}` in group `{group}`: `*` is only allowed as the last character"
    )]
    Pattern { group: String, pattern: String },

    #[error("unknown malva option `stylesheet.malva.{0}`")]
    MalvaOption(String),
}

/// Converts a byte offset into a 1-based (line, column) pair.
pub(crate) fn line_col(src: &str, offset: usize) -> (usize, usize) {
    let before = &src[..offset.min(src.len())];
    let line = before.matches('\n').count() + 1;
    let column = before
        .rfind('\n')
        .map_or(before.len(), |i| before.len() - i - 1)
        + 1;
    (line, column)
}
