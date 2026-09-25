use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use annealer::{Config, Language, Profile, format};
use clap::{Parser, ValueEnum};

/// Reorder HTML/Vue attributes by semantic meaning.
#[derive(Parser)]
#[command(name = "anyl", version, about)]
struct Cli {
    /// Files to format. Reads stdin when omitted.
    files: Vec<PathBuf>,

    /// Overwrite files in place.
    #[arg(short, long, conflicts_with = "check")]
    write: bool,

    /// Exit with status 1 if any file would change.
    #[arg(short, long)]
    check: bool,

    /// Built-in profile name (`html`, `vue`) or path to a profile YAML file.
    /// Defaults to the profile matching each file's language.
    #[arg(short, long)]
    profile: Option<String>,

    /// Language of stdin input.
    #[arg(short, long, value_enum, default_value = "html")]
    language: LanguageArg,
}

#[derive(Clone, Copy, ValueEnum)]
enum LanguageArg {
    Html,
    Vue,
    Css,
    Scss,
}

impl From<LanguageArg> for Language {
    fn from(value: LanguageArg) -> Self {
        match value {
            LanguageArg::Html => Self::Html,
            LanguageArg::Vue => Self::Vue,
            LanguageArg::Css => Self::Css,
            LanguageArg::Scss => Self::Scss,
        }
    }
}

fn load_profile(spec: &str) -> Result<Profile, String> {
    if let Some(profile) = Profile::builtin(spec) {
        return Ok(profile);
    }
    let yaml = std::fs::read_to_string(spec).map_err(|error| format!("{spec}: {error}"))?;
    Profile::from_yaml(&yaml).map_err(|error| format!("{spec}: {error}"))
}

fn config_for(language: Language, profile: Option<&Profile>) -> Config {
    match profile {
        Some(profile) => Config::new(language, profile.clone()),
        None => Config::for_language(language),
    }
}

/// Returns whether the file changed.
fn process_file(path: &Path, cli: &Cli, profile: Option<&Profile>) -> Result<bool, String> {
    let language = Language::from_path(path)
        .ok_or_else(|| format!("{}: unsupported file type", path.display()))?;
    let input =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let output = format(&input, &config_for(language, profile))
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let changed = output != input;

    if cli.write {
        if changed {
            std::fs::write(path, &output)
                .map_err(|error| format!("{}: {error}", path.display()))?;
        }
    } else if cli.check {
        if changed {
            println!("{}", path.display());
        }
    } else {
        io::stdout()
            .write_all(output.as_bytes())
            .map_err(|error| error.to_string())?;
    }
    Ok(changed)
}

fn run(cli: &Cli) -> Result<bool, String> {
    let profile = cli.profile.as_deref().map(load_profile).transpose()?;

    if cli.files.is_empty() {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|error| error.to_string())?;
        let output = format(&input, &config_for(cli.language.into(), profile.as_ref()))
            .map_err(|error| error.to_string())?;
        if !cli.check {
            io::stdout()
                .write_all(output.as_bytes())
                .map_err(|error| error.to_string())?;
        }
        return Ok(output != input);
    }

    let mut any_changed = false;
    let mut failed = false;
    for path in &cli.files {
        match process_file(path, cli, profile.as_ref()) {
            Ok(changed) => any_changed |= changed,
            Err(message) => {
                eprintln!("error: {message}");
                failed = true;
            }
        }
    }
    if failed {
        return Err("some files could not be formatted".to_owned());
    }
    Ok(any_changed)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(changed) if cli.check && changed => ExitCode::from(1),
        Ok(_) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::from(2)
        }
    }
}
