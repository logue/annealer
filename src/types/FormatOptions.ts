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
};

/** Default `format` options. */
export const FormatOptions = {
  language: 'html'
} as const satisfies FormatOptions;
