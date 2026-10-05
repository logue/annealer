/** Input language. */
export type Language = 'html' | 'vue' | 'css' | 'scss' | 'sass' | 'less';

/** File extensions mapped to their language. */
export const Language: Readonly<Record<string, Language>> = {
  html: 'html',
  htm: 'html',
  vue: 'vue',
  css: 'css',
  scss: 'scss',
  sass: 'sass',
  less: 'less'
};
