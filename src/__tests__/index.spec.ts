import { describe, expect, test } from 'rstack/test';

import { builtinProfile, format, init, languageFromPath } from '@/index';

describe('format', () => {
  test('orders HTML attributes by default', () => {
    expect(format('<a title="Home" href="/" id="home">Home</a>')).toBe(
      '<a id="home" title="Home" href="/">Home</a>'
    );
  });

  test('uses the Vue profile for Vue input', () => {
    expect(
      format(
        '<template>\n<div @click="c" v-if="x" id="i"></div>\n</template>\n',
        {
          language: 'vue'
        }
      )
    ).toBe('<template>\n<div v-if="x" id="i" @click="c" />\n</template>\n');
  });

  test('orders stylesheet declarations', () => {
    expect(
      format('a { color: red; display: block; }\n', { language: 'css' })
    ).toBe('a {\n  display: block;\n  color: red;\n}\n');
  });

  test('accepts a profile as YAML text', () => {
    const profile = [
      'schemaVersion: 1',
      'name: custom',
      'groups:',
      '  - name: rest',
      '    fallback: true',
      '    sort: alphabetical'
    ].join('\n');
    expect(format('<a id="i" title="t" href="/">', { profile })).toBe(
      '<a href="/" id="i" title="t">'
    );
  });

  test('throws on invalid input and profiles', () => {
    expect(() => format('<a title="t')).toThrow(/unterminated/u);
    expect(() => format('<a>', { profile: 'schemaVersion: 2' })).toThrow(
      /profile/u
    );
  });

  test('init can be awaited before formatting', async () => {
    await init();
    expect(format('<br>')).toBe('<br />');
  });
});

describe('helpers', () => {
  test('languageFromPath', () => {
    expect(languageFromPath('src/App.vue')).toBe('vue');
    expect(languageFromPath('style.SCSS')).toBe('scss');
    expect(languageFromPath('README.md')).toBeUndefined();
    expect(languageFromPath('Makefile')).toBeUndefined();
  });

  test('builtinProfile returns YAML', () => {
    expect(builtinProfile('vue')).toMatch(/^schemaVersion: 1$/mu);
  });
});
