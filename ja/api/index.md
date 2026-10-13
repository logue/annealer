> For AI agents: the complete documentation index is available at /annealer/ja/llms.txt, the full documentation bundle is available at /annealer/ja/llms-full.txt.

# API 概要

annealer は 1 つの Rust コアを、いくつかの方法で呼び出せます。

| 入口              | 用途                                      | リファレンス                                           |
| --------------- | --------------------------------------- | ------------------------------------------------ |
| `anyl`          | コマンドラインや CI でのファイル整形                    | [CLI](/annealer/ja/api/cli.md)                   |
| `annealer`（npm） | WebAssembly 経由で Node.js、ブラウザー、バンドラーから利用 | [JavaScript API](/annealer/ja/api/javascript.md) |
| プロファイル YAML     | 属性順、レイアウト、スタイルシートのオプションのカスタマイズ          | [プロファイルリファレンス](/annealer/ja/api/profile.md)      |

Rust の関数 `annealer::format(input, &Config)` は純粋関数で、ファイル I/O を行いません。そのため同じコアが CLI でも WebAssembly でも動きます。
