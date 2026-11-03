---
description: 计划中的库专用配置文件，以及在项目依赖中发现它们的既定策略。
---

# 库配置文件

:::info 状态
计划中。下述发现策略已确定，但尚未实现。
:::

annealer 将支持特定于某个库（UI 组件库、CSS 框架）的规则设置。这些设置复用现有的配置文件机制，而不是增加代码分支：

- 属性分组，
- 按元素种类的 `layout.selfClosing`，
- 规范化选项，
- 类值顺序（参见[值排序](./value-ordering.md)）。

这些机制保持通用且由数据驱动，因此库配置文件可以扩展或覆盖它们。这很可能需要配置文件组合（`extends`）以及按标签或按组件的覆盖。

## 发现策略 {#discovery-policy}

### 通过包内的 `annealer.yaml` 检测 {#detected-by-annealer-yaml-in-the-package}

当包的根目录中含有 `annealer.yaml` 时，该包即提供库配置文件。该文件的存在是唯一的信号。annealer 不使用包名约定，也不根据包的内容进行猜测。

### 默认仅限直接依赖 {#direct-dependencies-only-by-default}

只扫描项目的直接依赖。只有当库作者显式选择加入时，才会包含间接依赖。例如，基于 Bootstrap 构建的 UI 组件库可以在自己的 `annealer.yaml` 中声明其携带 Bootstrap 的配置文件。

依赖树深处存在某个包，并不意味着项目的标记使用了它的类。

### 检测与应用相互独立 {#detection-and-application-are-separate}

检测只负责找出候选项并报告。只有当用户在其配置中显式指定时，配置文件才会被应用。

安装或更新依赖本身绝不能改变 annealer 的输出。

## Bootstrap

对于 Bootstrap，计划是携带一份草案配置文件，向 Bootstrap 项目（v6 正在开发中）提出这一机制，由 Bootstrap 自己掌管其类顺序。这首先需要一个稳定且有文档的配置文件格式。
