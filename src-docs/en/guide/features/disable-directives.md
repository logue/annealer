---
description: Comments that switch annealer off for a region or for the next line, in markup and in stylesheets.
---

# Disable directives

Some code must keep its exact form, for example a third-party snippet or a legacy order kept on purpose. Comments switch annealer off for that code.

```html
<!-- annealer-disable -->
<!-- annealer-enable -->
<!-- annealer-disable-next-line: [reason] -->
```

```css
/* annealer-disable */
/* annealer-enable */
/* annealer-disable-next-line: [reason] */
```

## Rules

- **Block comments only.** `//` is not supported, because CSS has no line comments.
- **`annealer-disable`** covers everything after it, up to `annealer-enable` or the end of the document. At the top of a document, it leaves the file unchanged.
- **`annealer-disable-next-line`** covers the constructs that start on the next line:
  - a start tag (the whole tag, even if it spans several lines, and a `<style>` element's content), or
  - a CSS statement (a declaration or a whole rule).
- **The reason after `:` is optional** and ignored by annealer.
- **Covered code is kept verbatim and in place.** In a stylesheet, covered declarations also stay where they are while their neighbors are sorted.
- **Scope:** markup comments apply to markup (and to whole `<style>` elements). CSS comments apply inside the stylesheet they appear in.
- **Unknown directives are errors.** A comment that starts with `annealer-` but is not a known directive (for example the typo `annealer-disabel`) fails the run instead of being ignored.

```html
<!-- annealer-disable-next-line: copied from the vendor's docs -->
<a href="/" id="vendor-link">Kept as written</a>
<a href="/" id="home">Reordered</a>
```

## Implementation note (stylesheets)

A covered declaration keeps its position and ends the run of declarations sorted around it, so nothing moves across it. A covered rule is left exactly as written, nested blocks included. malva's own ignore comment (`malva-ignore`) is not used: it rejects the `: reason` form.
