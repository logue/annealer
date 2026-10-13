---
description: Planned library-specific profiles, and the agreed policy for discovering them in a project's dependencies.
---

# Library profiles

:::info Status
Planned. The discovery policy below is decided; the implementation is not done yet.
:::

annealer will support rule settings specific to a library (a UI kit, a CSS framework). These reuse the existing profile mechanism instead of adding code branches:

- attribute groups,
- `layout.selfClosing` per element kind,
- normalization options,
- class value order (see [Value ordering](./value-ordering.md)).

The mechanisms are kept general and data-driven, so a library profile can extend or override them. This will likely need profile composition (`extends`) and per-tag or per-component overrides.

## Discovery policy

### Detected by `annealer.yaml` in the package

A package provides a library profile when its package root contains an `annealer.yaml`. The presence of that file is the only signal. annealer doesn't use package-name conventions and doesn't guess from the package contents.

### Direct dependencies only, by default

Only the project's direct dependencies are scanned. A transitive dependency is included only when a library author opts in explicitly. For example, a UI kit built on Bootstrap can declare in its own `annealer.yaml` that it carries Bootstrap's profile.

A package somewhere deep in the dependency tree doesn't mean the project's markup uses its classes.

### Detection and application are separate

Detection only finds candidates and reports them. A profile is applied only when the user names it explicitly in their config.

Installing or updating a dependency must never change annealer's output by itself.

## Bootstrap

For Bootstrap, the plan is to propose the mechanism to the Bootstrap project (v6 is in development) with a draft profile, so that Bootstrap owns its class order. This requires a stable, documented profile format first.
