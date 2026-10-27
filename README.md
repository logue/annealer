# Annearler

HTML (-ish) attribute and Stylesheet attribute formatter.

See [Documentation](https://logue.dev/annealer/). If you're using this with AI, please load [llms.txt](https://logue.dev/annealer/llms.txt).

> [!NOTE]
> Currently, it only supports HTML and Vue, but future plans include adding support for JSX, Svelte, Astro, and others.

![annealer](https://github.com/user-attachments/assets/12c29ebf-f112-4b3a-bda7-ad10c50f4b3d)

[![npm version](https://img.shields.io/npm/v/annealer.svg)](https://www.npmjs.com/package/annealer)
[![NPM Downloads](https://img.shields.io/npm/dm/annealer.svg?style=flat)](https://www.npmjs.com/package/annealer)
[![jsdelivr CDN](https://data.jsdelivr.com/v1/package/npm/annealer/badge)](https://www.jsdelivr.com/package/npm/annealer)
[![Open in unpkg](https://img.shields.io/badge/Open%20in-unpkg-blue)](https://uiwjs.github.io/npm-unpkg/#/pkg/annealer/file/README.md)
[![Open in Gitpod](https://shields.io/badge/Open%20in-Gitpod-green?logo=Gitpod)](https://gitpod.io/#https://github.com/logue/annealer)
[![X Follow](https://img.shields.io/twitter/follow/logue256?style=plastic)](https://twitter.com/logue256)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/logue?label=Sponsor&logo=github&color=ea4aaa)](https://github.com/sponsors/logue)

## Usage

### CLI

```bash
anyl src/App.vue                 # print the result
anyl --write src/**/*.vue        # format in place
anyl --check index.html          # exit 1 if a file would change
anyl --profile my-profile.yaml page.html
```

Supported inputs: `.html`, `.vue`, `.css`, `.scss`, `.sass`, `.less`.

### JavaScript

```ts
import { format } from 'annealer';

format('<a title="Home" href="/" id="home">Home</a>');
// => '<a id="home" title="Home" href="/">Home</a>'

format(source, { language: 'vue' });
format(source, { language: 'html', profile: yamlText });
```

## License

©2026 by Logue.
Licensed under the [MIT License](LICENSE).
