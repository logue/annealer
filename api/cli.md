> For AI agents: the complete documentation index is available at /annealer/llms.txt, the full documentation bundle is available at /annealer/llms-full.txt.

# CLI

```text
anyl [OPTIONS] [FILES]...
```

Formats each file and prints the result. Reads stdin when no file is given.

## Options

| Option                        | Description                                                                                                                                                                                                                                                                                                                  |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-w`, `--write`               | Overwrite files in place. Can't be combined with `--check`.                                                                                                                                                                                                                                                                  |
| `-c`, `--check`               | Print the paths of files that would change, and exit with status 1 if any would.                                                                                                                                                                                                                                             |
| `-p`, `--profile <PROFILE>`   | A built-in profile name (`html`, `vue`) or the path to a profile YAML file. Defaults to the built-in profile for each file's language.                                                                                                                                                                                       |
| `--class-order <CLASS_ORDER>` | Order the classes of static `class` attributes: a built-in class order name (`tailwind`) or the path to a class order YAML file. Overrides the profile's `classOrder`.                                                                                                                                                       |
| `--remove-duplicate-classes`  | Drop repeated classes in static `class` attributes, keeping the first. Works with or without `--class-order`. By default the class order's `removeDuplicates` decides (off in `tailwind`). `--keep-duplicate-classes` keeps them.                                                                                            |
| `--self-closing`              | Self-close empty elements and components in Vue templates (`layout.selfClosing.normal`/`component`, `vue/html-self-closing`): `<div />`, `<MyComp />`. Off in the built-in profiles. `--no-self-closing` turns it off. Void elements (`<br />`) follow `--void-self-closing`.                                                |
| `--void-self-closing`         | Self-close void elements: `<input />`, `<meta />` (`layout.selfClosing.void: always`, the built-in profiles' setting). `--no-void-self-closing` writes `<input>`, `<meta>` (`never`).                                                                                                                                        |
| `--expand-non-void`           | In plain HTML, give non-void elements written self-closing an end tag: `<div />` → `<div></div>` (`layout.selfClosing.expandNonVoid`). HTML ignores the slash, so `<div />` leaves the element open. Elements inside `<svg>` and `<math>` are left alone. Off in the built-in profiles. `--no-expand-non-void` turns it off. |
| `--content-newline`           | Put the content of multiline elements on lines of its own (`vue/multiline-html-element-content-newline`). Off in the built-in profiles. `--no-content-newline` turns it off.                                                                                                                                                 |
| `--print-width <PRINT_WIDTH>` | Line width of stylesheets and `<style>` blocks (`stylesheet.malva.printWidth`). Match your formatter: 80 for Prettier and Biome, 100 for oxfmt. Defaults to the profile's (80).                                                                                                                                              |
| `-l`, `--language <LANGUAGE>` | Language of stdin input: `html` (default), `vue`, `css`, `scss`, `sass`, `less`.                                                                                                                                                                                                                                             |
| `-h`, `--help`                | Show help.                                                                                                                                                                                                                                                                                                                   |
| `-V`, `--version`             | Show the version.                                                                                                                                                                                                                                                                                                            |

## Languages

The language of a file is taken from its extension:

| Extension                         | Language   | Default profile                   |
| --------------------------------- | ---------- | --------------------------------- |
| `.html`, `.htm`                   | `html`     | `html`                            |
| `.vue`                            | `vue`      | `vue`                             |
| `.css`, `.scss`, `.sass`, `.less` | stylesheet | `html` (its `stylesheet` section) |

Other extensions, including `.pug`, `.jade`, `.styl` and `.stylus`, are reported as unsupported without the file being read. A file with a `<template>` or `<style>` in Pug or Stylus (`lang="pug"`, `"jade"`, `"stylus"`, `"styl"`) is an error too; see [Limitations](/annealer/guide/start/limitations.md#supported-input).

## Exit codes

| Code | Meaning                                                                                                                                              |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `0`  | Success. With `--check`: no file would change.                                                                                                       |
| `1`  | With `--check`: at least one file would change.                                                                                                      |
| `2`  | Error: unreadable file, unsupported file type, invalid profile, invalid input (including Pug or Stylus blocks) or an unknown `annealer-*` directive. |

When several files are given, annealer formats all of them, reports each error, and exits with `2` if any failed.

## Examples

```bash
anyl src/App.vue                          # print the result
anyl --write src/**/*.vue                 # format in place
anyl --check index.html                   # use in CI
anyl --profile my-profile.yaml page.html  # custom profile
anyl --class-order tailwind src/**/*.vue  # order Tailwind classes
anyl --class-order tailwind --remove-duplicate-classes src/**/*.vue  # and drop repeated ones
anyl --print-width 100 src/**/*.vue  # wrap stylesheets at 100, like oxfmt
anyl --self-closing --content-newline src/**/*.vue  # also apply the optional layout rules
cat App.vue | anyl --language vue         # stdin
```
