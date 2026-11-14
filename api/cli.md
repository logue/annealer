> For AI agents: the complete documentation index is available at /annealer/llms.txt, the full documentation bundle is available at /annealer/llms-full.txt.

# CLI

```text
anyl [OPTIONS] [FILES]...
```

Formats each file and prints the result. Reads stdin when no file is given.

## Options

| Option                        | Description                                                                                                                                                            |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-w`, `--write`               | Overwrite files in place. Can't be combined with `--check`.                                                                                                            |
| `-c`, `--check`               | Print the paths of files that would change, and exit with status 1 if any would.                                                                                       |
| `-p`, `--profile <PROFILE>`   | A built-in profile name (`html`, `vue`) or the path to a profile YAML file. Defaults to the built-in profile for each file's language.                                 |
| `--class-order <CLASS_ORDER>` | Order the classes of static `class` attributes: a built-in class order name (`tailwind`) or the path to a class order YAML file. Overrides the profile's `classOrder`. |
| `-l`, `--language <LANGUAGE>` | Language of stdin input: `html` (default), `vue`, `css`, `scss`, `sass`, `less`.                                                                                       |
| `-h`, `--help`                | Show help.                                                                                                                                                             |
| `-V`, `--version`             | Show the version.                                                                                                                                                      |

## Languages

The language of a file is taken from its extension:

| Extension                         | Language   | Default profile                   |
| --------------------------------- | ---------- | --------------------------------- |
| `.html`, `.htm`                   | `html`     | `html`                            |
| `.vue`                            | `vue`      | `vue`                             |
| `.css`, `.scss`, `.sass`, `.less` | stylesheet | `html` (its `stylesheet` section) |

Other extensions are reported as unsupported.

## Exit codes

| Code | Meaning                                                                                                             |
| ---- | ------------------------------------------------------------------------------------------------------------------- |
| `0`  | Success. With `--check`: no file would change.                                                                      |
| `1`  | With `--check`: at least one file would change.                                                                     |
| `2`  | Error: unreadable file, unsupported file type, invalid profile, invalid input or an unknown `annealer-*` directive. |

When several files are given, annealer formats all of them, reports each error, and exits with `2` if any failed.

## Examples

```bash
anyl src/App.vue                          # print the result
anyl --write src/**/*.vue                 # format in place
anyl --check index.html                   # use in CI
anyl --profile my-profile.yaml page.html  # custom profile
anyl --class-order tailwind src/**/*.vue  # order Tailwind classes
cat App.vue | anyl --language vue         # stdin
```
