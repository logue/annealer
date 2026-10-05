import { FormatOptions } from '@/types/FormatOptions';
import { Language } from '@/types/Language';
import initWasm, {
  builtinProfile as builtinProfileWasm,
  format as formatWasm,
  initSync
} from '@/wasm/annealer.js';
import { wasmBase64 } from '@/wasm/binary';

export { FormatOptions } from '@/types/FormatOptions';
export { Language } from '@/types/Language';
export { Meta } from '@/types/Meta';

/** Built-in profile names. */
export type BuiltinProfile = 'html' | 'vue';

let ready = false;

const decode = (base64: string): Uint8Array<ArrayBuffer> => {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.codePointAt(i) ?? 0;
  }
  return bytes;
};

const ensureReady = (): void => {
  if (!ready) {
    initSync({ module: decode(wasmBase64) });
    ready = true;
  }
};

/**
 * Compiles the WebAssembly core ahead of the first `format` call without
 * blocking. Optional: `format` compiles it synchronously when needed.
 */
export const init = async (): Promise<void> => {
  if (!ready) {
    await initWasm({ module_or_path: decode(wasmBase64) });
    ready = true;
  }
};

/**
 * Reorders attributes (HTML, Vue) or declarations (CSS, SCSS, Sass, Less)
 * by semantic meaning.
 *
 * @throws {Error} when the input or the profile is invalid.
 */
export const format = (input: string, options: FormatOptions = {}): string => {
  ensureReady();
  return formatWasm(
    input,
    options.language ?? FormatOptions.language,
    options.profile
  );
};

/** Infers the language from a file name or path. */
export const languageFromPath = (path: string): Language | undefined => {
  const extension = /\.([^./\\]+)$/u.exec(path)?.[1]?.toLowerCase();
  return extension === undefined ? undefined : Language[extension];
};

/** The YAML text of a built-in profile, as a starting point for a custom one. */
export const builtinProfile = (name: BuiltinProfile): string => {
  ensureReady();
  const yaml = builtinProfileWasm(name);
  if (yaml === undefined) {
    throw new Error(`unknown built-in profile \`${name}\``);
  }
  return yaml;
};
