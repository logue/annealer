> For AI agents: the complete documentation index is available at /annealer/llms.txt, the full documentation bundle is available at /annealer/llms-full.txt.

# API overview

annealer has one Rust core and several ways to call it:

| Entry point      | For                                                        | Reference                                     |
| ---------------- | ---------------------------------------------------------- | --------------------------------------------- |
| `anyl`           | Formatting files from the command line or in CI            | [CLI](/annealer/api/cli.md)                   |
| `annealer` (npm) | Node.js, browsers and bundlers, through WebAssembly        | [JavaScript API](/annealer/api/javascript.md) |
| Profile YAML     | Customizing attribute order, layout and stylesheet options | [Profile reference](/annealer/api/profile.md) |

The Rust function `annealer::format(input, &Config)` is pure: it does no file I/O, which is why the same core runs in the CLI and in WebAssembly.
