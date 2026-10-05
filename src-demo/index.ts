import { type Language, Meta, builtinProfile, format, init } from '@/';

const SAMPLES: Record<Language, string> = {
  vue: `<template>
  <MyButton @click="submit" :disabled="busy" v-if="visible" class="primary" :class="{ busy }" id="submit" aria-label="Submit"></MyButton>
  <div
    title="Panel" v-for="item in items" :key="item.id">{{ item.name }}</div>
</template>

<style scoped>
.panel { color: red; -webkit-box-shadow: none; display: flex; box-shadow: none; position: relative; }
</style>
`,
  html: `<img onload="loaded()" data-z="1" data-a="2" width="80" src="logo.png" alt="Logo" class="logo" id="logo">
<button aria-expanded="false" aria-controls="menu" aria-label="Menu" role="button" type="button">Menu</button>
`,
  css: `.card { color: #333; margin: 0; -webkit-transition: opacity 0.2s; transition: opacity 0.2s; display: block; position: relative; }
`,
  scss: `.card {
  color: #333; display: block;
  &:hover { opacity: 0.8; position: relative; }
}
`,
  sass: `.card
  color: #333
  display: block
  &:hover
    opacity: 0.8
    position: relative
`,
  less: `@gap: 8px;
.card { color: #333; margin: @gap; display: block; }
`
};

const element = <T extends HTMLElement>(id: string): T =>
  document.getElementById(id) as T;

const language = element<HTMLSelectElement>('language');
const profile = element<HTMLSelectElement>('profile');
const profileEditor = element<HTMLDetailsElement>('profile-editor');
const profileYaml = element<HTMLTextAreaElement>('profile-yaml');
const input = element<HTMLTextAreaElement>('input');
const output = element<HTMLTextAreaElement>('output');
const error = element<HTMLParagraphElement>('error');

const render = (): void => {
  const spec =
    profile.value === 'custom' ? profileYaml.value : profile.value || undefined;
  try {
    output.value = format(input.value, {
      language: language.value as Language,
      profile: spec
    });
    error.textContent = '';
  } catch (error_) {
    output.value = '';
    error.textContent = String(error_);
  }
};

language.addEventListener('change', () => {
  input.value = SAMPLES[language.value as Language];
  render();
});
profile.addEventListener('change', () => {
  if (profile.value === 'custom') {
    if (profileYaml.value === '') {
      profileYaml.value = builtinProfile(
        language.value === 'vue' ? 'vue' : 'html'
      );
    }
    profileEditor.open = true;
  }
  render();
});
profileYaml.addEventListener('input', render);
input.addEventListener('input', render);

element('version').textContent = `v${Meta.version}`;
input.value = SAMPLES.vue;
await init();
render();
