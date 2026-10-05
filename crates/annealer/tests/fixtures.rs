//! Fixture tests: every `tests/fixtures/<name>.<ext>` is formatted with the
//! built-in profile for its language and compared with
//! `tests/fixtures/<name>.expected.<ext>`. Formatting the expected output
//! again must not change it.
//!
//! Set `ANNEALER_UPDATE_FIXTURES=1` to (re)write the expected files, then
//! review the diff.

use std::fs;
use std::path::{Path, PathBuf};

use annealer::{Config, Language, format};

fn fixtures() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut inputs: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| !stem.ends_with(".expected"))
        })
        .collect();
    inputs.sort();
    inputs
}

fn expected_path(input: &Path) -> PathBuf {
    let stem = input.file_stem().unwrap().to_str().unwrap();
    let extension = input.extension().unwrap().to_str().unwrap();
    input.with_file_name(format!("{stem}.expected.{extension}"))
}

#[test]
fn fixtures_match_expected_output() {
    let update = std::env::var_os("ANNEALER_UPDATE_FIXTURES").is_some();
    let inputs = fixtures();
    assert!(!inputs.is_empty(), "no fixtures found");

    let mut failures = Vec::new();
    for input_path in inputs {
        let name = input_path.file_name().unwrap().to_string_lossy();
        let language = Language::from_path(&input_path)
            .unwrap_or_else(|| panic!("{name}: unsupported extension"));
        let config = Config::for_language(language);
        let input = fs::read_to_string(&input_path).unwrap();
        let output = format(&input, &config).unwrap_or_else(|error| panic!("{name}: {error}"));

        let expected_path = expected_path(&input_path);
        if update {
            fs::write(&expected_path, &output).unwrap();
        } else {
            let expected = fs::read_to_string(&expected_path)
                .unwrap_or_else(|error| panic!("{}: {error}", expected_path.display()));
            if output != expected {
                failures.push(format!("{name}: output differs from the expected file"));
            }
        }

        let again = format(&output, &config).unwrap();
        if again != output {
            failures.push(format!("{name}: formatting is not idempotent"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
