---
title: API 概要
description: annealer の入口である anyl CLI、JavaScript API、YAML プロファイル形式の概要。
---

# API 概要

annealer は 1 つの Rust コアを、いくつかの方法で呼び出せます。

| 入口              | 用途                                                         | リファレンス                             |
| ----------------- | ------------------------------------------------------------ | ---------------------------------------- |
| `anyl`            | コマンドラインや CI でのファイル整形                         | [CLI](./cli.md)                          |
| `annealer`（npm） | WebAssembly 経由で Node.js、ブラウザー、バンドラーから利用   | [JavaScript API](./javascript.md)        |
| プロファイル YAML | 属性順、レイアウト、スタイルシートのオプションのカスタマイズ | [プロファイルリファレンス](./profile.md) |

Rust の関数 `annealer::format(input, &Config)` は純粋関数で、ファイル I/O を行いません。そのため同じコアが CLI でも WebAssembly でも動きます。
