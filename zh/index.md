> For AI agents: the complete documentation index is available at /annealer/zh/llms.txt, the full documentation bundle is available at /annealer/zh/llms-full.txt.

# Annealer

按语义排列属性

> 依据可替换的 YAML 配置文件，按语义对 HTML/Vue 属性和样式表声明进行排序。

[简介](/guide/start/introduction) | [GitHub](https://github.com/logue/annealer)

## Features

- [🧭 **基于语义的属性顺序**](/guide/concepts/attribute-order): 按角色（定义、状态、补充含义、ARIA、数据、事件）对属性分组，而不是按字母顺序或书写顺序。
- [🧩 **vue/attributes-order 的超集**](/guide/concepts/attribute-order): Vue 配置文件沿用 eslint-plugin-vue 的默认顺序，并增加同名聚合、ARIA 子排序、纯 HTML 和样式表支持。
- [🎨 **通过 malva 处理样式表**](/guide/concepts/stylesheets): CSS、SCSS、Sass 和 Less 的属性顺序交由 malva 处理，带厂商前缀的声明放在最前面。
- [📄 **可替换的 YAML 配置文件**](/api/profile): 顺序和布局规则是数据而非代码。配置文件由 JSON Schema 校验，未知键会被拒绝。
- [🦀 **Rust 核心、CLI 与 WebAssembly**](/api/): 不做文件 I/O 的纯 Rust 函数、anyl CLI，以及提供同步 format() API 的 npm 包。
- [🛑 **禁用指令**](/guide/features/disable-directives): 通过 annealer-disable、annealer-enable 和 annealer-disable-next-line 注释，让指定代码保持原样。
