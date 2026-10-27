import { describe, expect, test } from 'rstack/test';

import {
  builtinClassOrder,
  builtinProfile,
  format,
  init,
  languageFromPath
} from '@/index';

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
    ).toBe('<template>\n<div v-if="x" id="i" @click="c"></div>\n</template>\n');
  });

  test('turns on the optional layout rules', () => {
    const input =
      '<template>\n<div>a\nb</div><MyComp></MyComp><br>\n</template>\n';
    // Void elements are always self-closed.
    expect(format(input, { language: 'vue' })).toBe(
      '<template>\n<div>a\nb</div><MyComp></MyComp><br />\n</template>\n'
    );
    expect(
      format(input, {
        language: 'vue',
        selfClosing: true,
        contentNewline: true
      })
    ).toBe('<template>\n<div>\n  a\nb\n</div><MyComp /><br />\n</template>\n');
  });

  test('removes duplicate classes when asked', () => {
    expect(format('<p class="b a b">')).toBe('<p class="b a b">');
    expect(format('<p class="b a b">', { removeDuplicateClasses: true })).toBe(
      '<p class="b a">'
    );
    expect(
      format('<p class="p-4 m-2 p-4">', {
        classOrder: 'tailwind',
        removeDuplicateClasses: true
      })
    ).toBe('<p class="m-2 p-4">');
  });

  test('sets the line width of stylesheets', () => {
    const input =
      'a { width: calc(100% - var(--sidebar-width) - var(--gutter-width) - var(--scrollbar-width)); }\n';
    expect(format(input, { language: 'css' })).toContain('calc(\n');
    expect(format(input, { language: 'css', printWidth: 100 })).toBe(
      'a {\n  width: calc(100% - var(--sidebar-width) - var(--gutter-width) - var(--scrollbar-width));\n}\n'
    );
    expect(() => format(input, { language: 'css', printWidth: 0 })).toThrow();
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

  test('orders classes with a class order', () => {
    const input = '<p class="p-3 m-4 lg:p-4 flex card">';
    expect(format(input)).toBe(input);
    expect(format(input, { classOrder: 'tailwind' })).toBe(
      '<p class="card m-4 flex p-3 lg:p-4">'
    );
    expect(() => format(input, { classOrder: 'name: x' })).toThrow(
      /class order|schemaVersion/u
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

  test('builtinClassOrder returns YAML', () => {
    expect(builtinClassOrder('tailwind')).toMatch(/^name: tailwind$/mu);
  });
});
