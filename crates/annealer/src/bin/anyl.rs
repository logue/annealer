use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use annealer::{ClassOrder, Config, Language, Profile, format};
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

    /// Order the classes of static `class` attributes: a built-in class
    /// order name (`tailwind`) or path to a class order YAML file.
    #[arg(long)]
    class_order: Option<String>,

    /// Self-close empty elements and components in Vue templates
    /// (`vue/html-self-closing`): `<div />`, `<MyComp />`. Off in the built-in
    /// profiles. Void elements (`<br />`) follow `--void-self-closing`.
    #[arg(long, overrides_with = "no_self_closing")]
    self_closing: bool,

    /// Do not self-close empty elements and components.
    #[arg(long, overrides_with = "self_closing")]
    no_self_closing: bool,

    /// Self-close void elements: `<input />`, `<meta />`
    /// (`selfClosing.void: always`, the built-in profiles' setting).
    #[arg(long, overrides_with = "no_void_self_closing")]
    void_self_closing: bool,

    /// Write void elements without a slash: `<input>`, `<meta>`
    /// (`selfClosing.void: never`).
    #[arg(long, overrides_with = "void_self_closing")]
    no_void_self_closing: bool,

    /// In plain HTML, give non-void elements written self-closing an end tag:
    /// `<div />` -> `<div></div>` (`selfClosing.expandNonVoid`). HTML ignores
    /// the slash, so `<div />` leaves the element open. Elements inside
    /// `<svg>` and `<math>` are left alone. Off in the built-in profiles.
    #[arg(long, overrides_with = "no_expand_non_void")]
    expand_non_void: bool,

    /// Leave non-void elements written self-closing alone.
    #[arg(long, overrides_with = "expand_non_void")]
    no_expand_non_void: bool,

    /// Put the content of multiline elements on lines of its own
    /// (`vue/multiline-html-element-content-newline`). Off in the built-in
    /// profiles.
    #[arg(long, overrides_with = "no_content_newline")]
    content_newline: bool,

    /// Do not put the content of multiline elements on lines of its own.
    #[arg(long, overrides_with = "content_newline")]
    no_content_newline: bool,

    /// Drop repeated classes in static `class` attributes, keeping the first.
    /// Works with or without `--class-order`. By default the class order's
    /// `removeDuplicates` decides (off in `tailwind`).
    #[arg(long, overrides_with = "keep_duplicate_classes")]
    remove_duplicate_classes: bool,

    /// Keep repeated classes.
    #[arg(long, overrides_with = "remove_duplicate_classes")]
    keep_duplicate_classes: bool,

    /// Line width of stylesheets and `<style>` blocks
    /// (`stylesheet.malva.printWidth`). Match your formatter: 80 for Prettier
    /// and Biome, 100 for oxfmt. Defaults to the profile's (80).
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    print_width: Option<u16>,

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
    Sass,
    Less,
}

impl From<LanguageArg> for Language {
    fn from(value: LanguageArg) -> Self {
        match value {
            LanguageArg::Html => Self::Html,
            LanguageArg::Vue => Self::Vue,
            LanguageArg::Css => Self::Css,
            LanguageArg::Scss => Self::Scss,
            LanguageArg::Sass => Self::Sass,
            LanguageArg::Less => Self::Less,
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

fn load_class_order(spec: &str) -> Result<ClassOrder, String> {
    if let Some(class_order) = ClassOrder::builtin(spec) {
        return Ok(class_order);
    }
    let yaml = std::fs::read_to_string(spec).map_err(|error| format!("{spec}: {error}"))?;
    ClassOrder::from_yaml(&yaml).map_err(|error| format!("{spec}: {error}"))
}

/// `Some(true)` for `--flag`, `Some(false)` for `--no-flag`, else `None`.
fn switch(on: bool, off: bool) -> Option<bool> {
    (on || off).then_some(on)
}

/// What every file shares: the profile, class order and layout options.
struct Options {
    profile: Option<Profile>,
    class_order: Option<ClassOrder>,
    self_closing: Option<bool>,
    void_self_closing: Option<bool>,
    expand_non_void: Option<bool>,
    content_newline: Option<bool>,
    remove_duplicate_classes: Option<bool>,
    print_width: Option<usize>,
}

fn config_for(language: Language, options: &Options) -> Config {
    let config = match &options.profile {
        Some(profile) => Config::new(language, profile.clone()),
        None => Config::for_language(language),
    };
    let config = match &options.class_order {
        Some(class_order) => config.with_class_order(Some(class_order.clone())),
        None => config,
    };
    config
        .with_self_closing(options.self_closing)
        .with_void_self_closing(options.void_self_closing)
        .with_expand_non_void(options.expand_non_void)
        .with_content_newline(options.content_newline)
        .with_remove_duplicate_classes(options.remove_duplicate_classes)
        .with_print_width(options.print_width)
}

/// Returns whether the file changed.
fn process_file(path: &Path, cli: &Cli, options: &Options) -> Result<bool, String> {
    let language = Language::from_path(path)
        .ok_or_else(|| format!("{}: unsupported file type", path.display()))?;
    let input =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let output = format(&input, &config_for(language, options))
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
    let options = Options {
        profile: cli.profile.as_deref().map(load_profile).transpose()?,
        class_order: cli
            .class_order
            .as_deref()
            .map(load_class_order)
            .transpose()?,
        self_closing: switch(cli.self_closing, cli.no_self_closing),
        void_self_closing: switch(cli.void_self_closing, cli.no_void_self_closing),
        expand_non_void: switch(cli.expand_non_void, cli.no_expand_non_void),
        content_newline: switch(cli.content_newline, cli.no_content_newline),
        remove_duplicate_classes: switch(cli.remove_duplicate_classes, cli.keep_duplicate_classes),
        print_width: cli.print_width.map(usize::from),
    };

    if cli.files.is_empty() {
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .map_err(|error| error.to_string())?;
        let output = format(&input, &config_for(cli.language.into(), &options))
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
        match process_file(path, cli, &options) {
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
