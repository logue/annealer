> For AI agents: the complete documentation index is available at /annealer/llms.txt, the full documentation bundle is available at /annealer/llms-full.txt.

# Getting started

annealer has three entry points that share the same Rust core:

- **`anyl`**: the command-line tool.
- **`annealer` (npm)**: a JavaScript API backed by WebAssembly.
- **`annealer` (Rust crate)**: the pure `format()` function.

## CLI

Build the CLI from the repository (the npm package and crate are not published yet):

```bash
cargo install --path crates/annealer
```

Then format files:

```bash
anyl src/App.vue                 # print the result
anyl --write "src/**/*.vue"      # format in place
anyl --check index.html          # exit 1 if a file would change
anyl --profile my-profile.yaml page.html
```

Supported inputs: `.html`, `.htm`, `.vue`, `.css`, `.scss`, `.sass`, `.less`. The language is detected from the extension, and each language uses its matching built-in profile unless `--profile` is given. See [CLI](/annealer/api/cli.md) for all options.

## JavaScript

```ts
import { format } from 'annealer';

format('<a title="Home" href="/" id="home">Home</a>');
// => '<a id="home" title="Home" href="/">Home</a>'

format(source, { language: 'vue' });
format(source, { language: 'html', profile: yamlText });
```

`format()` is synchronous. The WebAssembly binary is embedded in the package, so no loader or bundler setup is needed. See [JavaScript API](/annealer/api/javascript.md).

## Example

Input:

```html
<nav aria-label="Main" class="nav" id="nav">
  <a href="/docs" class="nav__link" aria-current="page">Docs</a>
</nav>
<input
  required
  placeholder="you@example.com"
  name="email"
  type="email"
  id="email"
  class="form__input"
  aria-describedby="email-help"
/>
```

Output with the `html` profile:

```html
<nav id="nav" class="nav" aria-label="Main">
  <a class="nav__link" href="/docs" aria-current="page">Docs</a>
</nav>
<input
  id="email"
  class="form__input"
  placeholder="you@example.com"
  type="email"
  name="email"
  required
  aria-describedby="email-help"
/>
```

- `id` and `class` come first, then supplementary (`placeholder`), semantic (`type`), key-value (`name`) and state (`required`) attributes, then ARIA.
- The multiline tag gets one attribute per line, and the closing bracket goes on its own line.
- The void element `<input>` becomes `<input />`.

## Profiles

Two profiles are built in:

| Profile | Used for                                   | Summary                                                    |
| ------- | ------------------------------------------ | ---------------------------------------------------------- |
| `html`  | `.html`, `.css`, `.scss`, `.sass`, `.less` | Semantic order for plain HTML.                             |
| `vue`   | `.vue`                                     | `vue/attributes-order` tiers plus annealer's sub-ordering. |

To customize, start from a built-in profile (`builtinProfile('vue')` in JavaScript, or `crates/annealer/profiles/*.yaml`) and pass your YAML with `--profile` or the `profile` option. See [Profile reference](/annealer/api/profile.md).

## Using with formatters and linters

Run annealer **before** your formatter and linter (oxfmt, Prettier, oxlint, Rslint, …):

1. **annealer** reorders attributes, classes and declarations, and may split a start tag over several lines.
2. **The formatter** then decides the final whitespace and line breaks of the reordered code.
3. **The linter** runs last and checks (or `--fix`es) the formatted result.

Run in the other order, annealer changes code the formatter already formatted (moved attributes, split tags), so the result isn't formatted any more and the check keeps failing.

### Chain the steps with `run-s`

We recommend chaining the steps with `run-s` from [npm-run-all2](https://github.com/bcomnes/npm-run-all2) rather than with `&&`:

- The steps run one after another, in the order written, and stop at the first failure.
- It doesn't depend on the shell that runs the script, so it behaves the same on every OS.
- Each step is a script of its own, so it can also run alone (`pnpm run format:annealer`).

```bash
pnpm add -D npm-run-all2
```

With oxfmt and oxlint:

```json
{
  "scripts": {
    "format": "run-s format:annealer format:oxfmt format:oxlint",
    "format:annealer": "anyl --write .",
    "format:oxfmt": "oxfmt",
    "format:oxlint": "oxlint --fix",
    "check": "run-s check:annealer check:oxfmt check:oxlint",
    "check:annealer": "anyl --check .",
    "check:oxfmt": "oxfmt --check",
    "check:oxlint": "oxlint"
  }
}
```

With Prettier and Rslint, limited to `src/`:

```json
{
  "scripts": {
    "format": "run-s format:annealer format:prettier format:rslint",
    "format:annealer": "anyl --write \"src/**/*.{vue,html,css,scss}\"",
    "format:prettier": "prettier --write .",
    "format:rslint": "rslint --fix",
    "check": "run-s check:annealer check:prettier check:rslint",
    "check:annealer": "anyl --check \"src/**/*.{vue,html,css,scss}\"",
    "check:prettier": "prettier --check .",
    "check:rslint": "rslint"
  }
}
```

Notes:

- Name the steps explicitly, as above. `run-s format:*` also works, but runs the matching scripts in the order they are written in package.json, which is easy to break when editing.
- Quote glob patterns in package.json (`\"src/**/*.vue\"`) so that `anyl` expands them itself. See [Files](/annealer/api/cli.md#files) for directories, globs, exclusions and ignore files.
- Run `check` in CI: it fails as soon as one tool would change something.
- Turn off rules and plugins that reorder the same things, or the tools fight each other: `prettier-plugin-tailwindcss` and `eslint-plugin-tailwindcss` (`classnames-order`) when you use a [class order](/annealer/guide/features/class-order.md), and a CSS property-order rule when your profile has `stylesheet.declarationOrder`. `vue/attributes-order` agrees with the built-in `vue` profile, which refines its tiers.
