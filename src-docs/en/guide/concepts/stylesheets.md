---
description: How annealer orders declarations in CSS, SCSS, Sass and Less through malva, where vendor-prefixed declarations go, and the opt-in style attribute support.
---

# Stylesheets

annealer doesn't implement CSS property ordering itself. It delegates to [malva](https://github.com/g-plane/malva), which already has a configurable `declarationOrder`. The `stylesheet` section of a profile controls this.

Formatted inputs:

- `<style>` blocks in HTML and Vue, including `<style lang="scss">`, `lang="less"` and `lang="sass"`.
- Standalone `.css`, `.scss`, `.sass` (indented syntax) and `.less` files.

If a profile has no `stylesheet` section, stylesheets are left untouched.

## Declaration order

Both built-in profiles use:

```yaml
stylesheet:
  declarationOrder: smacss
  vendorPrefix: start
```

- `declarationOrder`: `smacss` (important, flow-affecting properties first), `concentric` (from outside the box model inward), or `alphabetical`.
- `declarationOrderGroupBy`: how declarations are split into independently sorted runs: `non-declaration` (default) or `non-declaration-and-empty-line`.

```css
/* Input */
body {
  color: #222;
  margin: 0;
  font-family: system-ui, sans-serif;
  display: flex;
}

/* Output (smacss) */
body {
  display: flex;
  margin: 0;
  color: #222;
  font-family: system-ui, sans-serif;
}
```

## Vendor prefixes: `vendorPrefix: start` {#vendor-prefixes-vendorprefix-start}

Vendor-prefixed declarations (`-webkit-…`, `-moz-…`) are treated as separate from standard properties. They are dialects, and many were never standardized. With `start`, they go **before** the other declarations of the same run, in plain alphabetical order, with no grouping by property.

Why `start` and not `end`: a prefixed property is often an alias of the standard one, and the later declaration wins. Putting prefixes last could let a dialect override the standard value (for example, `-webkit-border-radius` with two values). `start` keeps the standard property last, as Autoprefixer's output does.

malva has no such option (it sorts by the unprefixed name). annealer re-parses malva's output and moves only the prefixed declarations. Runs match malva's sort units. A run with a comment between its declarations is left alone. The option applies only when `declarationOrder` is set. `alongside` (the default when not set) keeps malva's behavior.

## Static `style` attributes (opt-in) {#static-style-attributes-opt-in}

`stylesheet.styleAttribute: true` also orders the declarations inside static `style` attributes. It is off in both built-in profiles.

- The value stays on one line.
- CSS strings use the quote opposite to the attribute's quote.
- A missing trailing `;` stays missing.
- Bound `:style` is never touched.
- Values are left as written when they are unquoted, contain template syntax, comments, `{}`, `<` or `&`, or don't parse as plain declarations.

## Other malva options

Other malva options go under `stylesheet.malva` (for example `printWidth`, `quotes`, `hexCase`). Unknown keys are rejected, so a typo isn't silently ignored. See the [Profile reference](../../api/profile.md#stylesheet).
