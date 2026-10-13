---
title: API Overview
description: Entry points to annealer, the anyl CLI, the JavaScript API, and the YAML profile format.
---

# API overview

annealer has one Rust core and several ways to call it:

| Entry point      | For                                                        | Reference                         |
| ---------------- | ---------------------------------------------------------- | --------------------------------- |
| `anyl`           | Formatting files from the command line or in CI            | [CLI](./cli.md)                   |
| `annealer` (npm) | Node.js, browsers and bundlers, through WebAssembly        | [JavaScript API](./javascript.md) |
| Profile YAML     | Customizing attribute order, layout and stylesheet options | [Profile reference](./profile.md) |

The Rust function `annealer::format(input, &Config)` is pure: it does no file I/O, which is why the same core runs in the CLI and in WebAssembly.
