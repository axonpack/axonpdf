# @axonpdf/font

The font store behind `Font` in `@axonpdf/react`: the families you register, the emoji source and
the hyphenation callback. Every copy of this package in an app, and `@axonpdf/react`, share one
store, so a font registered through any of them is seen by all.

You only need this package if your code imports the font store on its own. In a React app,
`import { Font } from "@axonpdf/react"` gives you the same store.

## Install

```sh
npm install @axonpdf/font
```

The packages aren't on npm yet. Until they are, build them from this repo (see the root README).

## Example

```ts
import { sharedFontStore as Font } from "@axonpdf/font";

Font.register({
  family: "Lato",
  fonts: [
    { src: "fonts/Lato-Regular.ttf" },
    { src: "fonts/Lato-Bold.ttf", fontWeight: "bold" },
    { src: "fonts/Lato-Italic.ttf", fontStyle: "italic" },
  ],
});
Font.registerHyphenationCallback((word) => [word]); // no hyphenation
```

Every export is listed in [REFERENCE.md](REFERENCE.md). The project's [README](../../README.md)
covers the other packages.
