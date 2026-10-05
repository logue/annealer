# Annearler (WIP)

HTML (-ish) attribute and Stylesheet attribute formatter.

![annealer](https://github.com/user-attachments/assets/12c29ebf-f112-4b3a-bda7-ad10c50f4b3d)

[![X Follow](https://img.shields.io/twitter/follow/logue256?style=plastic)](https://x.com/logue256)
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
