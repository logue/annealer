// Generates the built-in `tailwind` class order profile and its test corpus
// from the installed `tailwindcss` package.
//
// annealer never runs Tailwind: the profile is plain data. Tailwind's own
// order (the one `eslint-plugin-tailwindcss` `classnames-order` and
// `prettier-plugin-tailwindcss` apply) comes from its compiler, so this script
// records what that compiler does for the default theme:
//
// - `propertyOrder`: Tailwind's internal CSS property order.
// - `variants`: the variant order (`getVariantOrder`).
// - `rules`: for each utility, the properties it sets, compressed into
//   patterns (`p-*`, `text-{number}`, exact names, …).
//
// It also writes a corpus of class lists sorted by Tailwind itself, which the
// Rust tests compare against.
//
// Usage: node scripts/generate-tailwind-order.mjs

import { readFileSync, writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';

import { __unstable__loadDesignSystem } from 'tailwindcss';

const require = createRequire(import.meta.url);
const root = fileURLToPath(new URL('..', import.meta.url));
const crate = `${root}crates/annealer`;
const tailwindVersion = JSON.parse(
  readFileSync(require.resolve('tailwindcss/package.json'), 'utf8')
).version;

const ds = await __unstable__loadDesignSystem('@import "tailwindcss";', {
  base: root,
  async loadStylesheet(id, base) {
    const path = require.resolve(
      id === 'tailwindcss' ? 'tailwindcss/index.css' : id
    );
    return { path, base, content: readFileSync(path, 'utf8') };
  }
});

// --- Property order -------------------------------------------------------

/** Tailwind's property order list is internal; read it from the bundle. */
const propertyOrder = (() => {
  const source = readFileSync(require.resolve('tailwindcss'), 'utf8');
  const start = source.indexOf('["container-type","pointer-events"');
  if (start === -1) {
    throw new Error('property order not found; update this script');
  }
  return JSON.parse(source.slice(start, source.indexOf(']', start) + 1));
})();

// --- Utilities ------------------------------------------------------------

/**
 * The properties a candidate sets, mirroring Tailwind's sort input: the
 * ranked properties (`order`) and the number of declarations (`count`).
 */
const signatureOf = name => {
  for (const candidate of ds.parseCandidate(name)) {
    if (candidate.variants.length > 0 || candidate.important) continue;
    const nodes = ds.compileAstNodes(candidate);
    if (nodes.length === 0) continue;
    const { order, count } = nodes[0].propertySort;
    const properties = order.map(index => propertyOrder[index]);
    return {
      candidate,
      key: `${order.join(',')}#${count}`,
      properties,
      // Declarations outside the ranked list (custom properties, `@property`
      // descriptors) still count; `count` defaults to the property count.
      count: properties.length === count ? undefined : count
    };
  }
  return undefined;
};

const NUMBER = /^\d+(?:\.\d+)?$/u;
const PERCENT = /^\d+(?:\.\d+)?%$/u;

/** Probes for arbitrary values whose type changes the utility. */
const ARBITRARY = {
  'arbitrary-color': '[#123456]',
  'arbitrary-length': '[10px]',
  'arbitrary-number': '[1.5]',
  'arbitrary-percentage': '[50%]',
  'arbitrary-url': '[url(x.png)]'
};
const VALUE_SETS = {
  'arbitrary-color': [
    '[#*]',
    '[rgb(*)]',
    '[rgba(*)]',
    '[hsl(*)]',
    '[hsla(*)]',
    '[hwb(*)]',
    '[lab(*)]',
    '[lch(*)]',
    '[oklab(*)]',
    '[oklch(*)]',
    '[color-mix(*)]',
    '[color:*]'
  ],
  'arbitrary-length': [
    '[*px]',
    '[*rem]',
    '[*em]',
    '[*vh]',
    '[*vw]',
    '[*dvh]',
    '[*svh]',
    '[*lvh]',
    '[*ch]',
    '[*ex]',
    '[calc(*)]',
    '[length:*]'
  ],
  'arbitrary-number': ['[number:*]'],
  'arbitrary-percentage': ['[*%]', '[percentage:*]'],
  'arbitrary-url': ['[url(*)]', '[image:*]']
};

/** signature key → { properties, count, patterns } */
const rules = new Map();
const addPattern = (signature, pattern) => {
  let rule = rules.get(signature.key);
  if (!rule) {
    rule = {
      properties: signature.properties,
      count: signature.count,
      patterns: []
    };
    rules.set(signature.key, rule);
  }
  if (!rule.patterns.includes(pattern)) rule.patterns.push(pattern);
};

const classList = ds.getClassList().map(([name]) => name);
/** functional root → signature key → { signature, values } */
const functional = new Map();
for (const name of classList) {
  const signature = signatureOf(name);
  if (!signature) continue;
  const { candidate } = signature;
  if (candidate.kind === 'static' || candidate.value === null) {
    addPattern(signature, name);
    continue;
  }
  const groups = functional.get(candidate.root) ?? new Map();
  functional.set(candidate.root, groups);
  const group = groups.get(signature.key) ?? { signature, values: [] };
  groups.set(signature.key, group);
  group.values.push(candidate.value.value);
}

const isColorGroup = group =>
  group.values.some(value => /^(?:red|blue|black|white)(?:-|$)/u.test(value));

for (const [utility, groups] of functional) {
  const all = [...groups.values()];
  // The group unknown values fall into: colors (custom theme colors are the
  // most common unknown value), else the largest group.
  const fallback =
    all.find(group => isColorGroup(group)) ??
    all.toSorted((a, z) => z.values.length - a.values.length)[0];
  addPattern(fallback.signature, `${utility}-*`);
  for (const group of all) {
    if (group === fallback) continue;
    for (const value of group.values) {
      if (NUMBER.test(value)) {
        addPattern(group.signature, `${utility}-{number}`);
      } else if (PERCENT.test(value)) {
        addPattern(group.signature, `${utility}-*%`);
      } else {
        addPattern(group.signature, `${utility}-${value}`);
      }
    }
  }
  {
    for (const [set, probe] of Object.entries(ARBITRARY)) {
      const signature = signatureOf(`${utility}-${probe}`);
      if (signature && signature.key !== fallback.signature.key) {
        addPattern(signature, `${utility}-{${set}}`);
      }
    }
  }
}

// --- Variants -------------------------------------------------------------

const variantProbes = [];
for (const variant of ds.getVariants()) {
  if (variant.values.length === 0 && !variant.isArbitrary) {
    variantProbes.push([variant.name, variant.name]);
    continue;
  }
  for (const value of variant.values) {
    const name = variant.hasDash
      ? `${variant.name}-${value}`
      : variant.name + value;
    variantProbes.push([name, name]);
  }
  if (variant.isArbitrary) {
    const separator = variant.hasDash ? '-' : '';
    variantProbes.push([
      `${variant.name}${separator}[x]`,
      `${variant.name}${separator}*`
    ]);
  }
}
variantProbes.push(['[&_p]', '[*]']);

/** Whether Tailwind generates CSS for a utility behind this variant. */
const isValidVariant = probe =>
  ds.getClassOrder([`${probe}:p-4`])[0][1] !== null;

const parsedVariants = [];
for (const [probe, pattern] of variantProbes) {
  const parsed = ds.parseVariant(probe);
  if (parsed && isValidVariant(probe)) parsedVariants.push([pattern, parsed]);
}
const variantOrder = ds.getVariantOrder();
/** rank → patterns */
const ranks = new Map();
for (const [pattern, parsed] of parsedVariants) {
  const rank = variantOrder.get(parsed);
  if (rank === undefined) continue;
  const patterns = ranks.get(rank) ?? [];
  if (!patterns.includes(pattern)) patterns.push(pattern);
  ranks.set(rank, patterns);
}
const variants = [...ranks]
  .toSorted(([a], [z]) => a - z)
  .map(([, patterns]) => patterns);

// --- YAML -----------------------------------------------------------------

const quote = value => JSON.stringify(value);
const list = values => `[${values.map(value => quote(value)).join(', ')}]`;

const lines = [
  '# yaml-language-server: $schema=../schema/class-order.schema.json',
  `# Generated by scripts/generate-tailwind-order.mjs from tailwindcss ${tailwindVersion}`,
  '# (default theme). Do not edit by hand; run the script again instead.',
  '#',
  "# Approximates Tailwind CSS's official class order, which",
  '# eslint-plugin-tailwindcss (`classnames-order`) and',
  '# prettier-plugin-tailwindcss apply. Classes are ordered by variants, then by',
  '# the properties they set (`propertyOrder`), then by the number of',
  '# properties (more first), then by name. Classes no rule matches go first.',
  '# Utilities and theme values a project adds itself are not known.',
  'schemaVersion: 1',
  'name: tailwind',
  `description: Tailwind CSS ${tailwindVersion.split('.').slice(0, 2).join('.')} class order (approximation of the official order).`,
  'unknown: start',
  'removeDuplicates: false',
  'important: "!"',
  'arbitraryProperties: true',
  'variants:',
  '  separator: ":"',
  '  order:',
  ...variants.map(patterns =>
    patterns.length === 1
      ? `    - ${quote(patterns[0])}`
      : `    - ${list(patterns)}`
  ),
  'propertyOrder:',
  ...propertyOrder.map(property => `  - ${quote(property)}`),
  'values:',
  ...Object.entries(VALUE_SETS).map(
    ([name, values]) => `  ${name}: ${list(values)}`
  ),
  'rules:'
];
const sortedRules = [...rules.values()].toSorted((a, z) =>
  a.patterns[0] < z.patterns[0] ? -1 : 1
);
for (const rule of sortedRules) {
  lines.push(
    `  - match: ${list(rule.patterns.toSorted())}`,
    `    properties: ${list(rule.properties)}`,
    ...(rule.count === undefined ? [] : [`    count: ${rule.count}`])
  );
}
writeFileSync(`${crate}/class-orders/tailwind.yaml`, `${lines.join('\n')}\n`);

// --- Test corpus ----------------------------------------------------------

let seed = 1;
const random = () => {
  seed = (seed * 1_103_515_245 + 12_345) % 2_147_483_648;
  return seed / 2_147_483_648;
};
const pick = items => items[Math.floor(random() * items.length)];

const knownVariants = variantProbes
  .map(([probe]) => probe)
  .filter(probe => isValidVariant(probe));
const extras = [
  'card',
  'btn-primary',
  'js-toggle',
  'w-[37px]',
  'text-[#ff0000]',
  'text-[14px]',
  'bg-[url(a.png)]',
  'grid-cols-[1fr_2fr]',
  'p-4!',
  '!m-2',
  'w-1/2',
  'bg-red-500/50',
  'text-lg/7',
  '-mt-4',
  '[mask-type:luminance]'
];
const corpus = [];
for (let i = 0; i < 3000; i++) {
  const size = 2 + Math.floor(random() * 8);
  const classes = [];
  for (let j = 0; j < size; j++) {
    let name = random() < 0.1 ? pick(extras) : pick(classList);
    const variantCount = random() < 0.3 ? 1 + Math.floor(random() * 2) : 0;
    for (let k = 0; k < variantCount; k++)
      name = `${pick(knownVariants)}:${name}`;
    // Variant combinations Tailwind rejects (`not-*:*:`) would test nothing
    // but the rejection itself.
    const valid = ds.getClassOrder([name])[0][1] !== null;
    if ((valid || extras.includes(name)) && !classes.includes(name)) {
      classes.push(name);
    }
  }
  // prettier-plugin-tailwindcss: unknown classes first, otherwise by order.
  const sorted = ds
    .getClassOrder(classes)
    .map((entry, index) => [...entry, index])
    .toSorted(([, a, ai], [, z, zi]) => {
      if (a === z) return ai - zi;
      if (a === null) return -1;
      if (z === null) return 1;
      return a < z ? -1 : 1;
    })
    .map(([name]) => name);
  corpus.push(`${classes.join(' ')}\t${sorted.join(' ')}`);
}
writeFileSync(
  `${crate}/tests/class-order/tailwind.tsv`,
  `${corpus.join('\n')}\n`
);

console.log(
  `tailwind.yaml: ${rules.size} rules, ${variants.length} variant ranks, ${propertyOrder.length} properties; corpus: ${corpus.length} lists`
);
