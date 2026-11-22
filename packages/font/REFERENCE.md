# @axonpdf/font reference

## Exports

| Export | Kind |
| --- | --- |
| `default` | the `FontStore` class |
| `sharedFontStore` | the one `FontStore` the app shares; `Font` in `@axonpdf/react` |
| `fontWeightNumber(weight?)` | function |
| `isStandardFont(src)` | function |
| `FontStoreType`, `FontLoader` | types |
| `FontStyle`, `FontWeight`, `FontDescriptor`, `RemoteOptions`, `FontSourceOptions`, `FontInput`, `SingleLoad`, `BulkLoad`, `FontSource`, `FontFamily`, `EmojiSource`, `HyphenationCallback` | types |

`sharedFontStore` lives under a global key, so two installs or two bundles of this package still
register into the same store.

## FontStore

A new store starts with the three built-in families registered.

| Member | What it does |
| --- | --- |
| `register({ family, src, fontStyle?, fontWeight?, postscriptName?, method?, headers?, body? })` | Adds one face to `family`. |
| `register({ family, fonts: [...] })` | Adds several faces, each `{ src, fontStyle?, fontWeight?, ... }`. |
| `registerEmojiSource(source)` | Sets where emoji images come from: `{ url, format?, withVariationSelectors? }` fetches `<url><code points>.<format>` (`png` by default), and `{ builder(code), withVariationSelectors? }` returns the URL itself. Emoji are drawn as images. |
| `registerHyphenationCallback(callback)` | Sets the document-wide hyphenation: `callback(word, builtin)` returns the parts of `word`. `builtin(word)` is the default split. `(word) => [word]` turns hyphenation off. |
| `getFont({ fontFamily, fontStyle?, fontWeight? })` | The registered face nearest to the weight in that style. Throws `Font family not registered: <family>. Please register it calling Font.register() method.` |
| `load(descriptor)` | Loads that face's bytes. |
| `loadSource(source)` | Loads one registered face's bytes. Built-in fonts have none. |
| `reset()` | Forgets every loaded font's bytes and keeps the registrations. |
| `clear()` | Removes every family, the emoji source and the hyphenation callback. |
| `getRegisteredFonts()` | The families, by name, each with its faces. |
| `getRegisteredFontFamilies()` | The family names. |
| `getEmojiSource()` | The emoji source, or `null`. |
| `getHyphenationCallback()` | The hyphenation callback, or `null`. |
| `loader` | The function that fetches a font's bytes, `(src, { method, headers, body }) => Promise<Uint8Array>`. It uses `fetch` by default; `@axonpdf/react` replaces it on the shared store so that Node also reads file paths. A failed fetch throws `Failed to fetch <src>: <status>`. |
| `fontFamilies`, `emojiSource`, `hyphenationCallback` | The fields behind the getters. |

## Font sources

`src` can be:

- a built-in font's name, such as `Helvetica-Bold`;
- a URL, fetched with the face's `method`, `headers` and `body`;
- a `data:` URL;
- in Node, a file path or `file:` URL.

The engine reads TrueType and OpenType files and WOFF and WOFF2. It rejects font collections
(TTC) with `Font collection is not supported`.

## Built-in fonts

Courier, Helvetica and Times-Roman are registered as families, each with regular, bold, italic
and bold italic faces:

| Family | Faces |
| --- | --- |
| `Helvetica` | `Helvetica`, `Helvetica-Bold`, `Helvetica-Oblique`, `Helvetica-BoldOblique` |
| `Courier` | `Courier`, `Courier-Bold`, `Courier-Oblique`, `Courier-BoldOblique` |
| `Times-Roman` | `Times-Roman`, `Times-Bold`, `Times-Italic`, `Times-BoldItalic` |

A face's name also works as a `fontFamily` of its own. Built-in fonts are drawn from their metrics
and never embedded; every PDF viewer has them.

## Weights

`fontWeight` is a number or one of these names:

| Name | Weight |
| --- | --- |
| `thin`, `hairline` | 100 |
| `ultralight`, `extralight` | 200 |
| `light` | 300 |
| `normal` | 400 |
| `medium` | 500 |
| `semibold`, `demibold` | 600 |
| `bold` | 700 |
| `ultrabold`, `extrabold` | 800 |
| `heavy`, `black` | 900 |

`fontWeightNumber` turns a name into its number (400 for an unknown name). When text asks for a
weight that isn't registered, the engine picks a face the way browsers do: for 600 it tries
heavier faces first, for 300 lighter ones.

## isStandardFont

`isStandardFont(src)` is `true` when `src` names one of the twelve built-in faces.
