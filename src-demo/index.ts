import '@carbon/web-components/es/components/accordion/index.js';
import type CDSAccordionItem from '@carbon/web-components/es/components/accordion/accordion-item.js';
import '@carbon/web-components/es/components/checkbox/index.js';
import type CDSCheckbox from '@carbon/web-components/es/components/checkbox/checkbox.js';
import '@carbon/web-components/es/components/grid/index.js';
import '@carbon/web-components/es/components/notification/inline-notification.js';
import type CDSInlineNotification from '@carbon/web-components/es/components/notification/inline-notification.js';
import '@carbon/web-components/es/components/select/index.js';
import type CDSSelect from '@carbon/web-components/es/components/select/select.js';
import '@carbon/web-components/es/components/tag/tag.js';
import '@carbon/web-components/es/components/textarea/index.js';
import type CDSTextarea from '@carbon/web-components/es/components/textarea/textarea.js';

import { type Language, Meta, builtinProfile, format, init } from '@/';

// The linter's resolver ignores stylesheets; Rsbuild resolves this import.
// rslint-disable-next-line import/no-unresolved
import './style.scss';

const SAMPLES: Record<Language, string> = {
  vue: `<template>
  <MyButton @click="submit" :disabled="busy" v-if="visible" class="primary" :class="{ busy }" id="submit" aria-label="Submit"></MyButton>
  <p class="text-sm p-3 hover:bg-gray-100 card flex mt-4 md:p-6 font-bold">Tailwind classes</p>
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

const element = (id: string): HTMLElement => {
  const found = document.getElementById(id);
  if (!found) throw new Error(`Missing element: ${id}`);
  return found;
};

const language = element('language') as CDSSelect;
const profile = element('profile') as CDSSelect;
const classOrder = element('class-order') as CDSSelect;
const removeDuplicateClasses = element(
  'remove-duplicate-classes'
) as CDSCheckbox;
const selfClosing = element('self-closing') as CDSCheckbox;
const noVoidSelfClosing = element('no-void-self-closing') as CDSCheckbox;
const expandNonVoid = element('expand-non-void') as CDSCheckbox;
const contentNewline = element('content-newline') as CDSCheckbox;
const profileEditor = element('profile-editor') as CDSAccordionItem;
const profileYaml = element('profile-yaml') as CDSTextarea;
const input = element('input') as CDSTextarea;
const output = element('output') as CDSTextarea;
const error = element('error') as CDSInlineNotification;

// Carbon's select treats an empty value as "unset", so the defaults use
// sentinel values that map back to `undefined` here.
const selectedProfile = (): string | undefined => {
  switch (profile.value) {
    case 'default':
      return undefined;
    case 'custom':
      return profileYaml.value;
    default:
      return profile.value;
  }
};

const render = (): void => {
  try {
    output.value = format(input.value, {
      language: language.value as Language,
      profile: selectedProfile(),
      classOrder: classOrder.value === 'keep' ? undefined : classOrder.value,
      // Unchecked keeps the profile's (or class order's) setting.
      removeDuplicateClasses: removeDuplicateClasses.checked || undefined,
      selfClosing: selfClosing.checked || undefined,
      voidSelfClosing: noVoidSelfClosing.checked ? false : undefined,
      expandNonVoid: expandNonVoid.checked || undefined,
      contentNewline: contentNewline.checked || undefined
    });
    error.subtitle = '';
    error.hidden = true;
  } catch (error_) {
    output.value = '';
    error.subtitle = String(error_);
    error.hidden = false;
  }
};

function sendHeight() {
  const height = document.documentElement.scrollHeight;
  window.parent.postMessage({ type: 'resize-iframe', height: height }, '*');
}

const applyTheme = (dark: boolean): void => {
  document.documentElement.classList.toggle('cds--white', !dark);
  document.documentElement.classList.toggle('cds--g100', dark);
};

// Regist Events

language.addEventListener('cds-select-selected', () => {
  input.value = SAMPLES[language.value as Language];
  render();
});
profile.addEventListener('cds-select-selected', () => {
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
classOrder.addEventListener('cds-select-selected', render);
removeDuplicateClasses.addEventListener('cds-checkbox-changed', render);
selfClosing.addEventListener('cds-checkbox-changed', render);
noVoidSelfClosing.addEventListener('cds-checkbox-changed', render);
expandNonVoid.addEventListener('cds-checkbox-changed', render);
contentNewline.addEventListener('cds-checkbox-changed', render);
profileYaml.addEventListener('input', render);
input.addEventListener('input', render);

const darkScheme = window.matchMedia('(prefers-color-scheme: dark)');
darkScheme.addEventListener('change', event => {
  applyTheme(event.matches);
});

window.addEventListener('load', sendHeight);
window.addEventListener('resize', sendHeight);

const observer = new MutationObserver(sendHeight);
observer.observe(document.body, { childList: true, subtree: true });

// Initialize

applyTheme(darkScheme.matches);
element('version').textContent = `v${Meta.version}`;
input.value = SAMPLES.vue;
await init();
render();
