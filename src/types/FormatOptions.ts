import type { Language } from '@/types/Language';

/** Options for `format`. */
export type FormatOptions = {
  /** Input language. */
  language?: Language;
  /**
   * A built-in profile name (`html`, `vue`) or the YAML text of a profile.
   * Defaults to the built-in profile for `language`.
   */
  profile?: string;
  /**
   * Order of the classes in static `class` attributes: a built-in class order
   * name (`tailwind`) or the YAML text of a class order. Overrides the
   * profile's `classOrder`; by default classes keep their order.
   */
  classOrder?: string;
  /**
   * Self-closes empty elements and components in Vue templates
   * (`layout.selfClosing.normal`/`component`, `vue/html-self-closing`):
   * `<div />`, `<MyComp />`. `false` turns that off. By default the profile's
   * setting is kept (off in the built-in profiles). Void elements (`<br />`)
   * always follow `layout.selfClosing.void`.
   */
  selfClosing?: boolean;
  /**
   * Puts the content of multiline elements on lines of its own
   * (`vue/multiline-html-element-content-newline`), with the rule's defaults
   * when the profile has no `layout.contentNewline`. `false` turns it off. By
   * default the profile's setting is kept (off in the built-in profiles).
   */
  contentNewline?: boolean;
  /**
   * Drops repeated classes in static `class` attributes, keeping the first.
   * Works with or without `classOrder`. `false` keeps them. By default the
   * class order's `removeDuplicates` is kept (off in `tailwind`, and off
   * without a class order).
   */
  removeDuplicateClasses?: boolean;
  /**
   * Line width of stylesheets and `<style>` blocks
   * (`stylesheet.malva.printWidth`). Match your formatter: 80 for Prettier and
   * Biome, 100 for oxfmt. By default the profile's setting is kept (80).
   */
  printWidth?: number;
};

/** Default `format` options. */
export const FormatOptions = {
  language: 'html'
} as const satisfies FormatOptions;
