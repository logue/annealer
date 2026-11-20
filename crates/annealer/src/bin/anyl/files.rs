//! Expands the CLI's file arguments, the way Prettier and oxfmt do.
//!
//! - A file is formatted as given.
//! - A directory is searched recursively for supported files.
//! - A glob pattern (`"src/**/*.vue"`) is expanded by `anyl` itself, so it
//!   works without shell support for `**`. Quote it to keep the shell out.
//! - A pattern starting with `!` (`"!src/legacy/**"`) excludes files.
//!
//! Directory and glob searches skip hidden files, `node_modules`, and what
//! `.gitignore` and `.anylignore` files list (gitignore syntax).

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use annealer::Language;
use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;

/// Ignore file read in every searched directory, like `.prettierignore`.
pub const IGNORE_FILE: &str = ".anylignore";

/// The files the arguments name, sorted and without duplicates.
pub struct Expanded {
    pub files: Vec<PathBuf>,
    /// Arguments that matched no file.
    pub unmatched: Vec<String>,
}

pub fn expand(args: &[String]) -> Result<Expanded, String> {
    let mut excludes = GlobSetBuilder::new();
    let mut includes = Vec::new();
    for arg in args {
        match arg.strip_prefix('!') {
            Some(pattern) => {
                excludes.add(glob(pattern)?);
            }
            None => includes.push(arg),
        }
    }
    let excludes = excludes
        .build()
        .map_err(|error| format!("invalid pattern: {error}"))?;

    let mut files = BTreeSet::new();
    let mut unmatched = Vec::new();
    for arg in includes {
        let found = if is_glob(arg) {
            search_glob(arg)?
        } else {
            let path = Path::new(arg);
            if path.is_dir() {
                search(path, &|_| true)
            } else if path.exists() {
                vec![clean(path)]
            } else {
                Vec::new()
            }
        };
        // A pattern whose files are all excluded still matched.
        if found.is_empty() {
            unmatched.push(arg.clone());
        }
        files.extend(
            found
                .into_iter()
                .filter(|file| !is_excluded(&excludes, file)),
        );
    }
    Ok(Expanded {
        files: files.into_iter().collect(),
        unmatched,
    })
}

fn is_glob(arg: &str) -> bool {
    arg.contains(['*', '?', '[', '{'])
}

fn glob(pattern: &str) -> Result<Glob, String> {
    GlobBuilder::new(pattern.strip_prefix("./").unwrap_or(pattern))
        // `*` stays within a path segment; `**` crosses them.
        .literal_separator(true)
        .build()
        .map_err(|error| format!("invalid pattern `{pattern}`: {error}"))
}

fn is_excluded(excludes: &GlobSet, file: &Path) -> bool {
    !excludes.is_empty() && excludes.is_match(file)
}

/// Supported files under the literal directory part of `pattern` that match it.
fn search_glob(pattern: &str) -> Result<Vec<PathBuf>, String> {
    let matcher = glob(pattern)?.compile_matcher();
    let base: PathBuf = Path::new(pattern)
        .components()
        .take_while(|component| !is_glob(&component.as_os_str().to_string_lossy()))
        .collect();
    let base = if base.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        base
    };
    if !base.is_dir() {
        return Ok(Vec::new());
    }
    Ok(search(&base, &|path| matcher.is_match(path)))
}

/// Supported files under `root` for which `keep` holds.
fn search(root: &Path, keep: &dyn Fn(&Path) -> bool) -> Vec<PathBuf> {
    WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .git_exclude(true)
        .git_global(false)
        // Honor `.gitignore` outside a Git repository too, as Prettier does.
        .require_git(false)
        .add_custom_ignore_filename(IGNORE_FILE)
        .filter_entry(|entry| entry.file_name() != "node_modules")
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .map(|entry| clean(entry.path()))
        .filter(|path| Language::from_path(path).is_some() && keep(path))
        .collect()
}

/// `./src/a.vue` → `src/a.vue`.
fn clean(path: &Path) -> PathBuf {
    path.components()
        .skip_while(|component| matches!(component, Component::CurDir))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dir(PathBuf);

    impl Dir {
        fn new(name: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("anyl-files-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&path);
            for file in [
                "index.html",
                "src/App.vue",
                "src/style.css",
                "src/main.ts",
                "src/legacy/Old.vue",
                "src/generated/Gen.vue",
                "node_modules/pkg/a.vue",
                ".hidden/b.vue",
            ] {
                let file = path.join(file);
                std::fs::create_dir_all(file.parent().unwrap()).unwrap();
                std::fs::write(file, "").unwrap();
            }
            std::fs::write(path.join(".gitignore"), "generated/\n").unwrap();
            Self(path)
        }

        fn expand(&self, args: &[&str]) -> (Vec<String>, Vec<String>) {
            let root = self.0.display();
            let args: Vec<String> = args
                .iter()
                .map(|arg| match arg.strip_prefix('!') {
                    Some(pattern) => format!("!{root}/{pattern}"),
                    None => format!("{root}/{arg}"),
                })
                .collect();
            let expanded = expand(&args).unwrap();
            let relative =
                |path: &PathBuf| path.strip_prefix(&self.0).unwrap().display().to_string();
            (
                expanded.files.iter().map(relative).collect(),
                expanded.unmatched,
            )
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn searches_directories() {
        let dir = Dir::new("dir");
        let (files, unmatched) = dir.expand(&["."]);
        // Not `main.ts`, `node_modules`, hidden or gitignored files.
        assert_eq!(
            files,
            [
                "index.html",
                "src/App.vue",
                "src/legacy/Old.vue",
                "src/style.css"
            ]
        );
        assert!(unmatched.is_empty());
    }

    #[test]
    fn expands_globs_and_exclusions() {
        let dir = Dir::new("glob");
        let (files, _) = dir.expand(&["src/**/*.vue"]);
        assert_eq!(files, ["src/App.vue", "src/legacy/Old.vue"]);
        let (files, _) = dir.expand(&["src/*.vue"]);
        assert_eq!(files, ["src/App.vue"]);
        let (files, _) = dir.expand(&["src/**/*.{vue,css}", "!src/legacy/**"]);
        assert_eq!(files, ["src/App.vue", "src/style.css"]);
    }

    #[test]
    fn reports_unmatched_arguments() {
        let dir = Dir::new("unmatched");
        let (files, unmatched) = dir.expand(&["src/**/*.scss", "missing.html", "index.html"]);
        assert_eq!(files, ["index.html"]);
        assert_eq!(unmatched.len(), 2);
    }

    #[test]
    fn honors_anylignore() {
        let dir = Dir::new("anylignore");
        std::fs::write(dir.0.join(IGNORE_FILE), "legacy/\n").unwrap();
        let (files, _) = dir.expand(&["src"]);
        assert_eq!(files, ["src/App.vue", "src/style.css"]);
    }
}
