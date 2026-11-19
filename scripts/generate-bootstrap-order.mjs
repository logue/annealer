// Generates the built-in `bootstrap` class order profile from the installed
// `bootstrap` package (`dist/css/bootstrap.css`).
//
// Bootstrap has no official class sorter, so this profile follows the same
// idea as the `tailwind` one: classes are ranked by where their CSS sits in
// the cascade. Bootstrap 6 puts everything into cascade layers
// (`@layer layout, content, forms, components, custom, helpers, utilities`),
// so the order is:
//
// - Classes of the layout, content, forms, components and helpers layers
//   (`container`, `row`, `btn`, `card`, `visually-hidden`, …), layer by layer.
//   Each layer is one entry of `propertyOrder` (`@layer components`); within a
//   layer, classes go by name, so a base class precedes its modifiers
//   (`btn btn-primary btn-lg`).
// - Utilities (`d-flex`, `mt-3`, `text-center`, …), by the CSS properties they
//   set, in the order the utilities layer first sets them.
// - Responsive and state variants (`md:d-flex`, `hover:…`) after the classes
//   without them, in the order Bootstrap emits them.
//
// Usage: node scripts/generate-bootstrap-order.mjs

import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const crate = `${root}crates/annealer`;
const bootstrapVersion = JSON.parse(
  readFileSync(`${root}node_modules/bootstrap/package.json`, 'utf8')
).version;
const css = readFileSync(`${root}node_modules/bootstrap/dist/css/bootstrap.css`, 'utf8');

// --- CSS parsing ----------------------------------------------------------

/**
 * Splits a stylesheet into a tree of `{ prelude, children }` blocks and
 * `{ property }` declarations. Enough for Bootstrap's compiled CSS.
 */
const parse = source => {
  const text = source.replaceAll(/\/\*[\s\S]*?\*\//gu, '');
  let index = 0;
  const block = () => {
    const children = [];
    let buffer = '';
    while (index < text.length) {
      const char = text[index++];
      if (char === '"' || char === "'") {
        const end = text.indexOf(char, index);
        buffer += char + text.slice(index, end + 1);
        index = end + 1;
      } else if (char === '\\') {
        buffer += char + text[index++];
      } else if (char === '{') {
        const prelude = buffer.trim();
        buffer = '';
        children.push({ prelude, children: block() });
      } else if (char === '}') {
        break;
      } else if (char === ';') {
        pushDeclaration(children, buffer);
        buffer = '';
      } else {
        buffer += char;
      }
    }
    pushDeclaration(children, buffer);
    return children;
  };
  const pushDeclaration = (children, buffer) => {
    const colon = buffer.indexOf(':');
    if (colon > 0) children.push({ property: buffer.slice(0, colon).trim() });
  };
  return block();
};

/** Splits on top-level commas. */
const splitSelectors = selector => {
  const parts = [];
  let depth = 0;
  let current = '';
  for (let index = 0; index < selector.length; index++) {
    const char = selector[index];
    if (char === '\\') {
      current += char + selector[++index];
      continue;
    }
    if (char === '(' || char === '[') depth++;
    if (char === ')' || char === ']') depth--;
    if (char === ',' && depth === 0) {
      parts.push(current.trim());
      current = '';
    } else {
      current += char;
    }
  }
  parts.push(current.trim());
  return parts;
};

/**
 * The class a selector starts with (unescaped) and whether the selector is
 * that class alone, or undefined.
 */
const leadingClass = selector => {
  const match = /^\.((?:\\.|[\w-])+)/u.exec(selector);
  return match
    ? { name: match[1].replaceAll(/\\(.)/gu, '$1'), alone: match[0] === selector }
    : undefined;
};

// --- Collecting classes ---------------------------------------------------

const LAYER_STATEMENT = /^@layer\s+([^{]+)$/u;
const layerOrder = (() => {
  const match = /@layer\s+([\w\s,-]+);/u.exec(css);
  if (!match) throw new Error('layer order not found; update this script');
  return match[1].split(',').map(name => name.trim());
})();
const CLASS_LAYERS = ['layout', 'content', 'forms', 'components', 'custom', 'helpers'];
for (const layer of [...CLASS_LAYERS, 'utilities']) {
  if (!layerOrder.includes(layer)) {
    throw new Error(`layer \`${layer}\` not found; update this script`);
  }
}

/** class → { layer, properties: Set } for classes without a variant. */
const classes = new Map();
/** Variants in the order they first appear. */
const variants = [];
/** Utility properties in the order they first appear. */
const utilityProperties = [];

const visit = (nodes, layer) => {
  for (const node of nodes) {
    if (!node.children) continue;
    const layerMatch = LAYER_STATEMENT.exec(node.prelude);
    if (layerMatch) {
      visit(node.children, layerMatch[1].trim());
      continue;
    }
    if (node.prelude.startsWith('@')) {
      visit(node.children, layer);
      continue;
    }
    if (!layer) continue;
    const declarations = node.children.filter(child => child.property).map(child => child.property);
    for (const selector of splitSelectors(node.prelude)) {
      const leading = leadingClass(selector);
      if (!leading) continue;
      const { name, alone } = leading;
      const separator = name.lastIndexOf(':');
      if (separator !== -1) {
        // `md:d-flex`, `hover:…`, `lg-down:…`
        const variant = name.slice(0, separator);
        if (!variants.includes(variant)) variants.push(variant);
        continue;
      }
      let entry = classes.get(name);
      if (!entry) {
        entry = { layer, properties: new Set() };
        classes.set(name, entry);
      }
      // Only a utility's own declarations rank it (`.d-flex { display }`),
      // not its descendants or pseudo-elements.
      if (entry.layer !== 'utilities' || layer !== 'utilities' || !alone) continue;
      for (const property of declarations) {
        entry.properties.add(property);
        if (!utilityProperties.includes(property)) utilityProperties.push(property);
      }
    }
  }
};
visit(parse(css), undefined);

// --- Rules ----------------------------------------------------------------

const propertyOrder = [
  ...CLASS_LAYERS.filter(layer => [...classes.values()].some(entry => entry.layer === layer)).map(
    layer => `@layer ${layer}`
  ),
  // Standard properties rank a utility; custom properties only when it sets
  // nothing else (`ratio-16x9` sets `--bs-ratio`).
  ...utilityProperties
];

/** signature key → { properties, count, names } */
const rules = new Map();
for (const [name, entry] of classes) {
  let properties;
  let count;
  if (entry.layer === 'utilities') {
    const all = [...entry.properties];
    const standard = all.filter(property => !property.startsWith('--'));
    properties = standard.length > 0 ? standard : all;
    if (properties.length === 0) continue;
    properties.sort((a, z) => propertyOrder.indexOf(a) - propertyOrder.indexOf(z));
    count = all.length === properties.length ? undefined : all.length;
  } else if (CLASS_LAYERS.includes(entry.layer)) {
    properties = [`@layer ${entry.layer}`];
  } else {
    continue;
  }
  const key = `${properties.join(',')}#${count}`;
  const rule = rules.get(key) ?? { properties, count, names: [] };
  rules.set(key, rule);
  rule.names.push(name);
}

/**
 * Compresses names that differ only in a trailing number (`mt-0` … `mt-5`)
 * into `mt-{number}` when every such name shares the rule.
 */
const NUMBERED = /^(.*?-)(\d+)$/u;
const ruleOf = new Map();
for (const rule of rules.values()) for (const name of rule.names) ruleOf.set(name, rule);
for (const rule of rules.values()) {
  const patterns = new Set();
  for (const name of rule.names) {
    const match = NUMBERED.exec(name);
    if (match) {
      const prefix = match[1];
      const siblings = [...classes.keys()].filter(other => {
        const otherMatch = NUMBERED.exec(other);
        return otherMatch && otherMatch[1] === prefix;
      });
      if (siblings.length > 1 && siblings.every(other => ruleOf.get(other) === rule)) {
        patterns.add(`${prefix}{number}`);
        continue;
      }
    }
    patterns.add(name);
  }
  rule.patterns = [...patterns].toSorted();
}

// --- YAML -----------------------------------------------------------------

const quote = value => JSON.stringify(value);
const list = values => `[${values.map(value => quote(value)).join(', ')}]`;

const lines = [
  '# yaml-language-server: $schema=../schema/class-order.schema.json',
  `# Generated by scripts/generate-bootstrap-order.mjs from bootstrap ${bootstrapVersion}`,
  '# (dist/css/bootstrap.css). Do not edit by hand; run the script again instead.',
  '#',
  '# Bootstrap has no official class order; this one follows the cascade.',
  '# Classes of the layout, content, forms, components and helpers layers come',
  '# first, layer by layer (one `@layer …` entry of `propertyOrder` each), by',
  '# name, so a base class precedes its modifiers (`btn btn-primary btn-lg`).',
  '# Utilities follow, by the properties they set, in the order the utilities',
  '# layer first sets them. Responsive and state variants (`md:`, `hover:`) come',
  '# after classes without them. Classes no rule matches go first.',
  'schemaVersion: 1',
  'name: bootstrap',
  `description: Bootstrap ${bootstrapVersion} class order (components, then utilities by property).`,
  'unknown: start',
  'removeDuplicates: false',
  'variants:',
  '  separator: ":"',
  '  order:',
  ...variants.map(variant => `    - ${quote(variant)}`),
  'propertyOrder:',
  ...propertyOrder.map(property => `  - ${quote(property)}`),
  'rules:'
];
const sortedRules = [...rules.values()].toSorted((a, z) =>
  a.patterns[0] < z.patterns[0] ? -1 : 1
);
for (const rule of sortedRules) {
  lines.push(
    `  - match: ${list(rule.patterns)}`,
    `    properties: ${list(rule.properties)}`,
    ...(rule.count === undefined ? [] : [`    count: ${rule.count}`])
  );
}
writeFileSync(`${crate}/class-orders/bootstrap.yaml`, `${lines.join('\n')}\n`);

console.log(
  `bootstrap.yaml: ${classes.size} classes, ${rules.size} rules, ${variants.length} variants, ${propertyOrder.length} properties`
);
