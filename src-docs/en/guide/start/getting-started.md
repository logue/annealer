---
description: Run annealer from the anyl CLI or from JavaScript, and choose a built-in or custom profile.
---

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
anyl --write src/**/*.vue        # format in place
anyl --check index.html          # exit 1 if a file would change
anyl --profile my-profile.yaml page.html
```

Supported inputs: `.html`, `.htm`, `.vue`, `.css`, `.scss`, `.sass`, `.less`. The language is detected from the extension, and each language uses its matching built-in profile unless `--profile` is given. See [CLI](../../api/cli.md) for all options.

## JavaScript

```ts
import { format } from 'annealer';

format('<a title="Home" href="/" id="home">Home</a>');
// => '<a id="home" title="Home" href="/">Home</a>'

format(source, { language: 'vue' });
format(source, { language: 'html', profile: yamlText });
```

`format()` is synchronous. The WebAssembly binary is embedded in the package, so no loader or bundler setup is needed. See [JavaScript API](../../api/javascript.md).

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

To customize, start from a built-in profile (`builtinProfile('vue')` in JavaScript, or `crates/annealer/profiles/*.yaml`) and pass your YAML with `--profile` or the `profile` option. See [Profile reference](../../api/profile.md).
