---
title: API 概览
description: annealer 的各个入口：anyl CLI、JavaScript API 和 YAML 配置文件格式。
---

# API 概览

annealer 只有一个 Rust 核心，但提供多种调用方式：

| 入口             | 用途                                                | 参考                              |
| ---------------- | --------------------------------------------------- | --------------------------------- |
| `anyl`           | 在命令行或 CI 中格式化文件                          | [CLI](./cli.md)                   |
| `annealer` (npm) | 通过 WebAssembly 在 Node.js、浏览器和打包工具中使用 | [JavaScript API](./javascript.md) |
| 配置文件 YAML    | 自定义属性顺序、布局和样式表选项                    | [配置文件参考](./profile.md)      |

Rust 函数 `annealer::format(input, &Config)` 是纯函数：它不做任何文件 I/O，因此同一个核心可以同时运行在 CLI 和 WebAssembly 中。
