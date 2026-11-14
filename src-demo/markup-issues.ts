/**
 * Lightweight well-formedness check for the demo's status banner.
 *
 * The formatter itself tolerates recoverable markup errors (unclosed
 * elements, stray end tags, duplicate attributes) and formats around them
 * without reporting what it skipped, so the demo scans the input itself to
 * tell "formatted" apart from "repaired and formatted".
 */

/** Void elements: no end tag expected. */
const VOID_ELEMENTS = new Set([
  'area',
  'base',
  'br',
  'col',
  'embed',
  'hr',
  'img',
  'input',
  'link',
  'meta',
  'param',
  'source',
  'track',
  'wbr'
]);

/** Elements whose content is raw text, not markup. */
const RAW_TEXT_ELEMENTS = new Set(['script', 'style']);

/** A start or end tag: `<name …>` / `</name>`, with quoted values kept intact. */
const TAG = /<(?<end>\/?)(?<name>[a-zA-Z][^\s/>]*)(?<attrs>(?:"[^"]*"|'[^']*'|[^"'>])*)>/gu;

/** An attribute inside a start tag, with an optional value. */
const ATTRIBUTE = /(?<name>[^\s"'=/]+)\s*(?:=\s*(?:"[^"]*"|'[^']*'|[^\s"'>]*))?/gu;

/** 1-based line number of `offset`, as text for the issue messages. */
const line = (source: string, offset: number): string =>
  String(source.slice(0, offset).split('\n').length);

/** Blanks out HTML comments, keeping offsets and line breaks intact. */
const withoutComments = (source: string): string =>
  source.replace(/<!--[\s\S]*?(?:-->|$)/gu, comment => comment.replace(/[^\n]/gu, ' '));

const duplicateAttributes = (attributes: string): string[] => {
  const seen: Set<string> = new Set();
  const duplicates: string[] = [];
  for (const match of attributes.matchAll(ATTRIBUTE)) {
    const name = match.groups?.name ?? '';
    if (seen.has(name) && !duplicates.includes(name)) duplicates.push(name);
    seen.add(name);
  }
  return duplicates;
};

/**
 * Finds recoverable markup problems in HTML or Vue template source:
 * unclosed elements, stray end tags, and duplicate attributes.
 *
 * @returns human-readable issue descriptions; empty when well-formed.
 */
export const findMarkupIssues = (input: string): string[] => {
  const source = withoutComments(input);
  const issues: string[] = [];
  const stack: { name: string; line: string }[] = [];

  // `exec` instead of `matchAll`: skipping raw-text content below moves
  // `TAG.lastIndex`, which `matchAll` would ignore (it iterates on a clone).
  TAG.lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = TAG.exec(source)) !== null) {
    const name = match.groups?.name ?? '';
    const attributes = match.groups?.attrs ?? '';
    const at = line(source, match.index);

    if (match.groups?.end === '/') {
      let found = -1;
      for (let index = stack.length - 1; index >= 0; index--) {
        if (stack[index]?.name.toLowerCase() === name.toLowerCase()) {
          found = index;
          break;
        }
      }
      if (found === -1) {
        issues.push(`stray </${name}> at line ${at}`);
        continue;
      }
      for (const skipped of stack.splice(found).slice(1)) {
        issues.push(`unclosed <${skipped.name}> at line ${skipped.line}`);
      }
      continue;
    }

    for (const attribute of duplicateAttributes(attributes)) {
      issues.push(`duplicate attribute \`${attribute}\` on <${name}> at line ${at}`);
    }
    if (VOID_ELEMENTS.has(name.toLowerCase()) || attributes.endsWith('/')) continue;
    if (RAW_TEXT_ELEMENTS.has(name.toLowerCase())) {
      // Skip raw content so a `<` inside a script or style block is not
      // mistaken for a tag; the end tag is consumed by the next iteration.
      const close = source.toLowerCase().indexOf(`</${name.toLowerCase()}`, TAG.lastIndex);
      if (close === -1) {
        issues.push(`unclosed <${name}> at line ${at}`);
        break;
      }
      TAG.lastIndex = close;
      stack.push({ name, line: at });
      continue;
    }
    stack.push({ name, line: at });
  }

  for (const open of stack) {
    issues.push(`unclosed <${open.name}> at line ${open.line}`);
  }
  return issues;
};
