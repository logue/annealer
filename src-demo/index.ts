import '@carbon/web-components/es/components/accordion/index.js';

import { findMarkupIssues } from './markup-issues';

import type CDSAccordionItem from '@carbon/web-components/es/components/accordion/accordion-item.js';
import '@carbon/web-components/es/components/grid/index.js';
import '@carbon/web-components/es/components/notification/inline-notification.js';
import type CDSInlineNotification from '@carbon/web-components/es/components/notification/inline-notification.js';
import '@carbon/web-components/es/components/select/index.js';
import type CDSSelectItem from '@carbon/web-components/es/components/select/select-item.js';
import type CDSSelect from '@carbon/web-components/es/components/select/select.js';
import '@carbon/web-components/es/components/tag/tag.js';
import '@carbon/web-components/es/components/textarea/index.js';
import type CDSTextarea from '@carbon/web-components/es/components/textarea/textarea.js';

import { type Language, Meta, builtinProfile, format, init } from '@/';

import './index.scss';

/** Sample flavors selectable in the Sample dropdown. */
type SampleVariant = 'default' | 'repairable' | 'invalid';

/**
 * Samples available per language. Each entry maps to a text file under
 * `samples/`: `default` to `{language}.txt`, the failure patterns to
 * `{language}.{variant}.txt`.
 */
const SAMPLE_VARIANTS: Record<Language, readonly SampleVariant[]> = {
  vue: ['default', 'repairable', 'invalid'],
  html: ['default', 'repairable', 'invalid'],
  css: ['default', 'invalid'],
  scss: ['default', 'invalid'],
  sass: ['default', 'invalid'],
  less: ['default', 'invalid']
};

const element = (id: string): HTMLElement => {
  const found = document.getElementById(id);
  if (!found) throw new Error(`Missing element: ${id}`);
  return found;
};

const language = element('language') as CDSSelect;
const sample = element('sample') as CDSSelect;
const profile = element('profile') as CDSSelect;
const classOrder = element('class-order') as CDSSelect;
const profileEditor = element('profile-editor') as CDSAccordionItem;
const profileYaml = element('profile-yaml') as CDSTextarea;
const input = element('input') as CDSTextarea;
const output = element('output') as CDSTextarea;
const status = element('status') as CDSInlineNotification;

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

const showStatus = (
  kind: 'success' | 'warning' | 'error',
  title: string,
  subtitle: string
): void => {
  status.setAttribute('kind', kind);
  status.title = title;
  status.subtitle = subtitle;
  status.hidden = false;
};

const render = (): void => {
  const inputLanguage = language.value as Language;
  try {
    output.value = format(input.value, {
      language: inputLanguage,
      profile: selectedProfile(),
      classOrder: classOrder.value === 'keep' ? undefined : classOrder.value
    });
  } catch (error) {
    output.value = '';
    showStatus('error', 'Could not format', String(error));
    return;
  }
  // The formatter recovers from markup problems without reporting them, so
  // scan the input to tell "formatted" apart from "repaired and formatted".
  const issues =
    inputLanguage === 'html' || inputLanguage === 'vue' ? findMarkupIssues(input.value) : [];
  if (issues.length > 0) {
    showStatus('warning', 'Repaired and formatted', `Recovered from: ${issues.join('; ')}.`);
  } else {
    showStatus('success', 'Formatted', 'No problems found.');
  }
};

// Sample sources live in `samples/*.txt` next to the page (plain text, so
// formatters leave the intentionally broken ones alone) and load on demand.
const sampleCache: Map<string, string> = new Map();

const loadSample = async (): Promise<void> => {
  const name = sample.value === 'default' ? language.value : `${language.value}.${sample.value}`;
  let text = sampleCache.get(name);
  if (text === undefined) {
    const response = await fetch(`samples/${name}.txt`);
    if (!response.ok) {
      throw new Error(`failed to load sample \`${name}.txt\` (HTTP ${String(response.status)})`);
    }
    text = await response.text();
    sampleCache.set(name, text);
  }
  input.value = text;
  render();
};

const applySample = (): void => {
  loadSample().catch((error: unknown) => {
    output.value = '';
    showStatus('error', 'Could not load the sample', String(error));
  });
};

/** Greys out the Sample entries the current language has no file for. */
const syncSampleOptions = (): void => {
  const variants = SAMPLE_VARIANTS[language.value as Language];
  for (const item of sample.querySelectorAll<CDSSelectItem>('cds-select-item')) {
    item.disabled = !variants.includes(item.value as SampleVariant);
  }
  if (!variants.includes(sample.value as SampleVariant)) {
    sample.value = 'default';
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
  syncSampleOptions();
  applySample();
});
sample.addEventListener('cds-select-selected', applySample);
profile.addEventListener('cds-select-selected', () => {
  if (profile.value === 'custom') {
    if (profileYaml.value === '') {
      profileYaml.value = builtinProfile(language.value === 'vue' ? 'vue' : 'html');
    }
    profileEditor.open = true;
  }
  render();
});
classOrder.addEventListener('cds-select-selected', render);
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
await init();
syncSampleOptions();
applySample();
