> For AI agents: the complete documentation index is available at /annealer/llms.txt, the full documentation bundle is available at /annealer/llms-full.txt.

# annealer

Attribute order by meaning

> Reorders HTML/Vue attributes and stylesheet declarations by semantic meaning, using swappable YAML profiles.

[Introduction](/guide/start/introduction) | [GitHub](https://github.com/logue/annealer)

## Features

- [🧭 **Semantic attribute order**](/guide/concepts/attribute-order): Attributes are grouped by role (definition, state, supplementary meaning, ARIA, data, events) instead of alphabetically or as typed.
- [🧩 **Superset of vue/attributes-order**](/guide/concepts/attribute-order): The Vue profile keeps eslint-plugin-vue's default order and adds same-name clustering, ARIA sub-ordering, plain HTML and stylesheets.
- [🎨 **Stylesheets via malva**](/guide/concepts/stylesheets): Property order for CSS, SCSS, Sass and Less is delegated to malva, with vendor-prefixed declarations placed first.
- [📄 **Swappable YAML profiles**](/api/profile): Orders and layout rules are data, not code. Profiles are validated by a JSON Schema and reject unknown keys.
- [🦀 **Rust core, CLI and WebAssembly**](/api/): A pure Rust function with no file I/O, the anyl CLI, and an npm package with a synchronous format() API.
- [🛑 **Disable directives**](/guide/features/disable-directives): annealer-disable, annealer-enable and annealer-disable-next-line comments keep selected code exactly as written.
