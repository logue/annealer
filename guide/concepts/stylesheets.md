> For AI agents: the complete documentation index is available at /annealer/llms.txt, the full documentation bundle is available at /annealer/llms-full.txt.

# Stylesheets

annealer doesn't define CSS property orders itself. It uses the orders of [malva](https://github.com/g-plane/malva)'s `declarationOrder`. The `stylesheet` section of a profile controls this.

**Only the order changes.** Declarations are moved exactly as written. Whitespace, indentation, quotes, color case, numbers, comments and blank lines are left alone: formatting is the job of a formatter such as Prettier or Oxfmt. malva is a formatter, but annealer uses it only to find out how property names sort, not to print the stylesheet.

Inputs:

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
- `declarationOrderGroupBy`: how declarations are split into independently sorted runs: `non-declaration-and-empty-line` (default; a blank line also ends a run) or `non-declaration` (declarations are sorted across blank lines, which stay where they are).

Declarations never move across another statement (a nested rule, `@include`, a Less mixin call, …) or a comment, so comments and the groups they label stay intact. A declaration covered by a [disable directive](/annealer/guide/features/disable-directives.md) or holding a nested block (Sass nested properties) also stays in place and ends the run.

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

## Vendor prefixes: `vendorPrefix: start` \{#vendor-prefixes-vendorprefix-start}

Vendor-prefixed declarations (`-webkit-…`, `-moz-…`) are treated as separate from standard properties. They are dialects, and many were never standardized. With `start`, they go **before** the other declarations of the same run, in plain alphabetical order, with no grouping by property.

Why `start` and not `end`: a prefixed property is often an alias of the standard one, and the later declaration wins. Putting prefixes last could let a dialect override the standard value (for example, `-webkit-border-radius` with two values). `start` keeps the standard property last, as Autoprefixer's output does.

malva has no such option (it sorts by the unprefixed name), so annealer moves the prefixed declarations itself, within the same runs. The option applies only when `declarationOrder` is set. `alongside` (the default when not set) keeps malva's order.

## Static `style` attributes (opt-in) \{#static-style-attributes-opt-in}

`stylesheet.styleAttribute: true` also orders the declarations inside static `style` attributes. It is off in both built-in profiles.

- Declarations are moved as written; spacing and a missing trailing `;` don't change.
- Bound `:style` is never touched.
- Values are left as written when they are unquoted, contain template syntax, `{}`, `<` or `&`, or don't parse as plain declarations.
