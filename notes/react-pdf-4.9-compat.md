# react-pdf 4.9.0 compatibility spec

`@axonpdf/react` is a drop-in replacement for `@react-pdf/renderer` 4.9.0. This file is the list of
everything that has to match. "Drop-in" may be claimed only when every row is `done`.

- **Status:** `todo`, `partial`, `done`, or `n/a` (with the reason in the row).
- **Fixture:** the document in `fixtures/` that proves the row against react-pdf's stored answer.
  A row is `done` only when its fixture passes.
- **Source:** paths in react-pdf's repository (`diegomura/react-pdf`) at commit `f765c95`, the
  4.9.0 tree the research read. `packages/...` is the code, `apps/site/content/docs/v4/...` the
  docs.
- **Rows are split when a fixture needs it.** A grouped row (for example a set of style keys) is
  split into one row per key as soon as one key is done before the others.
- We match react-pdf's public API and observed output. We never copy its code.

### A. Package shape

| Item | react-pdf 4.9.0 | Source | Status | Fixture |
|---|---|---|---|---|
| Name and peer | `@react-pdf/renderer`, peer `react` ^16.8 to ^19 | `packages/renderer/package.json` | todo |  |
| Node vs browser build | `main` plus a `browser` field map, no `exports` | `packages/renderer/package.json` | todo |  |
| Wrong-environment errors | Node build throws for `usePDF`, `PDFViewer`, `PDFDownloadLink`, `BlobProvider`; browser build throws for `renderToStream`, `renderToBuffer`, `renderToString`, `renderToFile`, `render` | `packages/renderer/src/node/index.js:6-26`, `packages/renderer/src/dom/index.js:9-33` | todo |  |
| Default export | object with every API and primitive (marked for removal) | `packages/renderer/src/node/index.js:37-51` | todo |  |
| Type shape | `export = ReactPDF` namespace; props as `ReactPDF.DocumentProps` etc. | `packages/renderer/index.d.ts:23-29` | todo |  |
| Sibling imports users write | `Style` from `@react-pdf/types`, plus `@react-pdf/stylesheet`, `/font`, `/tailwind`, `/math`, `/mermaid`, `/hyphenate` | `packages/types/style.d.ts:1`, `packages/types/font.d.ts` | todo |  |

### B. Render functions

| Item | react-pdf 4.9.0 | Source | Status | Fixture |
|---|---|---|---|---|
| `pdf(element?)` | returns `{ container, toBlob, toBuffer, toString, on, removeListener, updateContainer(el, cb?) }`; one shared reconciler | `packages/renderer/src/index.js:22-188` | todo |  |
| `isDirty()` | declared in types, not implemented | `packages/renderer/index.d.ts:786` vs `src/index.js:179-187` | todo |  |
| `instance.toBlob()` | `Promise<Blob>` of type `application/pdf` | `packages/renderer/src/index.js:104-126` | todo |  |
| `instance.toBuffer()` | returns a Node readable stream despite the name | `packages/renderer/src/index.js:128-134` | todo |  |
| `instance.toString()` | deprecated, uncompressed, warns in dev | `packages/renderer/src/index.js:142-166` | todo |  |
| `instance.on('change', fn)` | fires on every reconciler commit (`resetAfterCommit`) | `packages/renderer/src/index.js:23-26,168-177` | todo |  |
| `Document onRender` | called with `{ blob }` after `toBlob`, `{}` after `toBuffer` | `packages/renderer/src/index.js:98-102` | todo |  |
| `renderToStream(el)` | `Promise<NodeJS.ReadableStream>`; whole layout first, then pdfkit output | `packages/renderer/src/node/renderTo.js:10-14` | todo |  |
| `renderToFile(el, path, cb?)` | pipes to `fs.createWriteStream`; `render` is an alias | `packages/renderer/src/node/renderTo.js:21-34`, `node/index.js:28` | todo |  |
| `renderToBuffer(el)` | `Promise<Buffer>` | `packages/renderer/src/node/renderTo.js:40-49` | todo |  |
| `renderToString(el)` | deprecated, warns | `packages/renderer/src/node/renderTo.js:51-59` | todo |  |
| `createRenderer({ onChange })` | exported; typed as `PDFRenderer: any` | `packages/renderer/src/index.js:196`, `index.d.ts:782` | todo |  |
| `version` | package version string | `packages/renderer/src/index.js:11` | todo |  |
| Compression | `compress = true` inside `render()` | `packages/renderer/src/index.js:38,65-66` | todo |  |

### C. Browser components and hook

| Item | react-pdf 4.9.0 | Source | Status | Fixture |
|---|---|---|---|---|
| `usePDF({ document? })` | `[{ url, blob, loading, error }, update(doc)]`; one render at a time, newer replaces queued; revokes old object URLs | `packages/renderer/src/dom/usePDF.js:12-77` | todo |  |
| `PDFViewer` | iframe; props `title`, `style`, `className`, `children`, `innerRef`, `showToolbar = true` (appends `#toolbar=0/1`), rest spread on the iframe (`width`, `height`) | `packages/renderer/src/dom/PDFViewer.js:5-31` | todo |  |
| `PDFDownloadLink` | `<a download>`; `document`, `fileName = 'document.pdf'`, children node or `fn(instance)`, `onClick(event, instance)`, forwards ref | `packages/renderer/src/dom/PDFDownloadLink.js:5-50` | todo |  |
| `BlobProvider` | `document` plus `children(instance)` | `packages/renderer/src/dom/BlobProvider.js:5-16` | todo |  |

### D. StyleSheet and Font

| Item | react-pdf 4.9.0 | Source | Status | Fixture |
|---|---|---|---|---|
| `StyleSheet.create(s)` | identity | `packages/renderer/src/index.js:192-194` | todo |  |
| `Font.register` | `{ family, src, fontStyle?, fontWeight?, postscriptName?, method?, headers?, body? }` or `{ family, fonts: [...] }` | `packages/font/src/index.ts:123-150`, `packages/font/src/types.ts:33-56` | todo |  |
| Font sources | standard-font name, base64 data URL, URL (fetched with method, headers, body), Node file path; collections rejected; TTF and WOFF only | `packages/font/src/font-source.ts:58-83`, `apps/site/content/docs/v4/fonts.mdx:11` | todo |  |
| Weights | numbers or `thin` … `black`; nearest registered weight, as browsers pick | `packages/font/src/types.ts:10-25`, `apps/site/content/docs/v4/fonts.mdx:137` | todo |  |
| Built-in families | Courier, Helvetica, Times-Roman and their Bold/Oblique/Italic names; not embedded | `packages/font/src/index.ts:16-118`, `apps/site/content/docs/v4/fonts.mdx:79-92` | todo |  |
| `Font.registerHyphenationCallback(cb)` | `cb(word, builtin)` returns parts; `(w) => [w]` disables | `packages/font/src/index.ts:156-158`, `packages/textkit/src/layout/wrapWords.ts:46-62` | todo |  |
| `Font.registerEmojiSource` | `{ url, format?, withVariationSelectors? }` or `{ builder(code), withVariationSelectors? }`; emoji drawn as images | `packages/font/src/index.ts:152-154`, `packages/font/src/types.ts:58-69` | todo |  |
| `Font.getFont(descriptor)` | throws `Font family not registered: X. Please register it calling Font.register() method.` | `packages/font/src/index.ts:160-170` | todo |  |
| Other `Font` methods | `load`, `reset`, `clear`, `getRegisteredFonts`, `getRegisteredFontFamilies`, `getEmojiSource`, `getHyphenationCallback` | `packages/font/src/index.ts:172-202` | todo |  |

### E. Props every primitive takes

| Prop | react-pdf 4.9.0 | Source | Status | Fixture |
|---|---|---|---|---|
| `id` | target for `#id` links (destinations) | `packages/renderer/index.d.ts:79`, `apps/site/content/docs/v4/advanced/document-navigation.mdx:7-31` | todo |  |
| `style` | object or array; falsy entries skipped, merged left to right | `packages/renderer/index.d.ts:80`, `apps/site/content/docs/v4/styling.mdx:38-47` | todo |  |
| `fixed` | repeats the element on every page the `Page` wraps into | `packages/renderer/index.d.ts:81-85`, `apps/site/content/docs/v4/advanced/page-wrapping.mdx:76-96` | todo |  |
| `break` | starts a new page at the element | `packages/renderer/index.d.ts:86-91` | todo |  |
| `minPresenceAhead` | no break within n pt after the element | `packages/renderer/index.d.ts:92-96`, `apps/site/content/docs/v4/advanced/orphans-and-widows.mdx:11` | todo |  |
| `wrap` | default true; Image, Svg, Canvas, Note never wrap | `packages/layout/src/node/getWrap.ts:4-11` | todo |  |
| `debug` | draws the box, padding and margin | `packages/renderer/index.d.ts:127-131`, `packages/render/src/primitives/renderDebug.ts` | todo |  |
| `bookmark` | string or `{ title, top, left, zoom, fit, expanded }`; nests by tree depth | `packages/types/bookmark.d.ts:1-10`, `apps/site/content/docs/v4/advanced/document-navigation.mdx:33-66` | todo |  |
| `render` (View) | `({ pageNumber, subPageNumber }) => node` | `packages/renderer/index.d.ts:160-163` | todo |  |
| `render` (Text) | `({ pageNumber, totalPages, subPageNumber, subPageTotalPages })`; called twice, once with the total | `packages/renderer/index.d.ts:271-276`, `apps/site/content/docs/v4/advanced/dynamic-content.mdx:31-38`, `packages/layout/src/steps/resolvePagination.ts:247-254` | todo |  |

### F. Components and their own props

| Component | react-pdf 4.9.0 props | Source | Status | Fixture |
|---|---|---|---|---|
| `Document` | `style`, `title`, `author`, `subject`, `keywords`, `creator = 'react-pdf'`, `producer = 'react-pdf'`, `language`, `creationDate = new Date()`, `modificationDate`, `pdfVersion` (1.3 to 1.7ext3), `conformance` (PDF/A-1 to PDF/A-3b), `pageMode`, `pageLayout`, `ownerPassword`, `userPassword`, `permissions`, `onRender` | `packages/renderer/index.d.ts:47-66`, `packages/renderer/src/index.js:39-91`, `packages/types/pdf.d.ts`, `packages/types/node.d.ts:41-55` | todo |  |
| `Page` | `size` (A4 default; 4A0 to ID1, `[w]`, `[w, h]`, `{ width, height? }`, height left out means auto), `orientation`, `wrap = true`, `dpi = 72`, `debug`, `id`, `bookmark`, `style` (its padding is the page margin on every wrapped page), `layout` (experimental template), `experimentalPagination` | `packages/renderer/index.d.ts:107-136`, `packages/types/page.d.ts`, `packages/layout/src/page/getSize.ts:170` | todo |  |
| `View` | shared props, `wrap`, `debug`, `render` | `packages/renderer/index.d.ts:148-164` | todo |  |
| `Text` | shared props, `wrap`, `debug`, `render`, `hyphenationCallback`, `hyphenationPenalty` (100 justified, 600 otherwise, `Infinity` off), `orphans = 2`, `widows = 2`; nested `Text`/`Link` are inline runs; a string outside `Text` is dropped with a warning | `packages/renderer/index.d.ts:259-298`, `packages/renderer/src/renderer.js:43-63` | todo |  |
| `Link` | `href` or `src`, `#id` for internal, `hitSlop` (number or sides), `wrap`, `debug`; inside `Text` or around any primitive | `packages/renderer/index.d.ts:319-338`, `packages/types/hitslop.d.ts` | todo |  |
| `Image` | `src` or `source`: URL, Buffer, Blob, `{ data, format }`, `{ uri, method, headers, body, credentials }`, sync or async factory, Promise; `cache`, `srcSet`, `sizes`; PNG, JPEG (EXIF orientation), SVG | `packages/renderer/index.d.ts:174-211`, `packages/types/image.d.ts:1-37`, `packages/image/src/resolve.ts:109-137` | todo |  |
| `ImageBackground` | Image props plus `imageStyle` and children, as in React Native | `packages/renderer/index.d.ts:213-257` | todo |  |
| `Note` | string child; a PDF sticky-note annotation | `packages/renderer/index.d.ts:446-450` | todo |  |
| `Canvas` | `paint(painter, availableWidth, availableHeight)`; painter is chainable pdfkit: `dash clip save path fill font text rect scale moveTo lineTo stroke rotate circle lineCap opacity ellipse polygon restore lineJoin fontSize fillColor lineWidth translate miterLimit strokeColor fillOpacity roundedRect fillAndStroke strokeOpacity bezierCurveTo quadraticCurveTo linearGradient radialGradient` | `packages/renderer/index.d.ts:452-465`, `packages/render/src/primitives/renderCanvas.ts:4-38` | todo |  |
| SVG set | `Svg` (`width`, `height`, `viewBox`, `preserveAspectRatio`), `G`, `Path` (`d`), `Rect`, `Circle`, `Ellipse`, `Line`, `Polyline`, `Polygon`, `Text` (`x`, `y`), `Tspan`, `Defs`, `ClipPath`, `LinearGradient`, `RadialGradient`, `Stop`, `Marker`; presentation attributes `fill color stroke transform strokeDasharray opacity strokeWidth fillOpacity fillRule strokeOpacity textAnchor strokeLinecap strokeLinejoin visibility clipPath markerStart markerMid markerEnd dominantBaseline` | `packages/renderer/index.d.ts:467-695`, `packages/types/svg.d.ts` | todo |  |
| Form fields | `FieldSet` (`name`); `TextInput` (`align`, `multiline`, `password`, `noSpell`, `format`, `fontSize`, `maxLength`); `Checkbox` (`backgroundColor`, `borderColor`, `checked`, `onState`, `offState`, `xMark`); `Select`, `List` (`sort`, `edit`, `multiSelect`, `noSpell`, `select`); shared `name`, `required`, `noExport`, `readOnly`, `value`, `defaultValue` | `packages/renderer/index.d.ts:348-444` | todo |  |
| Primitive ids | `G SVG VIEW TEXT LINK PAGE NOTE PATH RECT LINE FIELD_SET TEXT_INPUT SELECT CHECKBOX LIST STOP DEFS IMAGE IMAGE_BACKGROUND TSPAN CANVAS CIRCLE ELLIPSE POLYGON DOCUMENT POLYLINE CLIP_PATH TEXT_INSTANCE FRAGMENT LINEAR_GRADIENT RADIAL_GRADIENT MARKER` | `packages/primitives/src/index.ts:1-32` | todo |  |

### G. Styles

| Group | react-pdf 4.9.0 | Source | Status | Fixture |
|---|---|---|---|---|
| Flex and position | `display` (`flex`, `none` only), `flexDirection`, `flexWrap`, `flexFlow`, `flex`, `flexGrow`, `flexShrink`, `flexBasis`, `alignItems`, `alignSelf`, `alignContent`, `justifyContent`, `justifySelf`, `gap`, `rowGap`, `columnGap`, `aspectRatio`, `zIndex`, `overflow: 'hidden'`, `position` (`absolute`, `relative`, `static`), `top/right/bottom/left` | `packages/stylesheet/src/types.ts:217-258`, `packages/stylesheet/src/resolve/flex.ts`, `layout.ts` | todo |  |
| Box | `width`, `height`, `min*`, `max*` (% allowed), `margin*` with `Horizontal`/`Vertical` and `auto`, `padding*` likewise | `packages/stylesheet/src/resolve/margins.ts`, `paddings.ts`, `dimensions.ts` | todo |  |
| Borders | `border` and per-side shorthands as strings (`'1pt solid red'`), per-side `Color/Style/Width`, per-corner radius (% allowed); styles `solid`, `dashed`, `dotted` | `packages/stylesheet/src/resolve/borders.ts`, `types.ts:13-40` | todo |  |
| Color | `color`, `backgroundColor`, `opacity`; hex, `rgb(a)`, `hsl(a)` | `packages/stylesheet/src/utils/colors.ts:1-27` | todo |  |
| Text | `fontFamily` (string or array), `fontSize`, `fontStyle`, `fontWeight`, `fontFeatureSettings` (array or object), `letterSpacing`, `lineHeight` (multiplier, unit or %), `textAlign`, `textDecoration` (+`Color`, `Style`), `textIndent`, `textTransform`, `textOverflow: 'ellipsis'` with `maxLines`, `verticalAlign` (`sub`, `super`), `direction`, `wordSpacing` (inherited) | `packages/stylesheet/src/resolve/text.ts`, `types.ts:316-407` | todo |  |
| Transform | `transform` string or array, `transformOrigin` (+X/Y), `gradientTransform` | `packages/stylesheet/src/resolve/transform.ts`, `types.ts:202` | todo |  |
| Image fit | `objectFit`, `objectPosition` (+X/Y) | `packages/stylesheet/src/resolve/positioning.ts` | todo |  |
| Floats | `float`, `clear`, `shapeOutside` (`circle()`, `ellipse()`, `polygon()`, `inset()`) | `packages/layout/src/steps/resolveFloats.ts`, `apps/site/content/docs/v4/floats.mdx` | todo |  |
| SVG paint | `fill`, `stroke`, `strokeWidth`, `strokeDasharray`, `strokeLinecap`, `strokeLinejoin`, `strokeOpacity`, `fillOpacity`, `fillRule`, `textAnchor`, `dominantBaseline`, `visibility`, `clipPath` | `packages/stylesheet/src/resolve/svg.ts` | todo |  |
| Units | `pt` (default), `in`, `mm`, `cm`, `px` (scaled by page `dpi`), `rem` (base 18), `vw`, `vh`, `%`; the docs list only pt/in/mm/cm/%/vw/vh | `packages/stylesheet/src/utils/units.ts:3-66`, `apps/site/content/docs/v4/styling.mdx:72-82` | todo |  |
| Media queries | `'@media max-width: 400': {...}`, `orientation: landscape`, matched against the page box | `packages/stylesheet/src/mediaQueries/index.ts:12-19`, `types.ts:519`, `apps/site/content/docs/v4/styling.mdx:52-68` | todo |  |
| Inheritance | `color fontFamily fontSize fontStyle fontWeight fontFeatureSettings letterSpacing opacity textDecoration textTransform lineHeight textAlign visibility wordSpacing`, plus `backgroundColor` into nested Text | `packages/layout/src/steps/resolveInheritance.ts:9-29` | todo |  |
| Defaults | `flexDirection: column`; Yoga border-box; `flexShrink` default 1 and **0 is coerced to 1**; `flex: n` means grow n, shrink 1, basis 0; `fontSize` 18; `color` black; `fontFamily` Helvetica, and Helvetica is appended as the last fallback | `packages/layout/src/node/setFlexDirection.ts:21`, `setFlexShrink.ts:11`, `packages/stylesheet/src/resolve/flex.ts:14-15`, `packages/layout/src/text/getAttributedString.ts:47-70` | todo |  |

### H. Text and pagination behaviour that decides layout parity

| Behaviour | react-pdf 4.9.0 | Source | Status | Fixture |
|---|---|---|---|---|
| Line breaking | Knuth-Plass (with a best-fit fallback) | `packages/textkit/src/engines/linebreaker/knuthPlass.ts:142` | todo |  |
| Hyphenation | on by default with en-us patterns; soft hyphens in a word replace the patterns | `packages/textkit/src/engines/wordHyphenation/index.ts:1-20` | todo |  |
| Font fallback | `fontFamily` array tried in order, then Helvetica | `packages/layout/src/text/getAttributedString.ts:66-70` | todo |  |
| Breakable set | View, Text, Link split; Image, Svg, Canvas, Note move whole | `apps/site/content/docs/v4/advanced/page-wrapping.mdx:25-26` | todo |  |
| Too-tall unbreakable | warns `Node of type X can't wrap between pages and it's bigger than available page height`; content is squeezed, not continued | `packages/layout/src/steps/resolvePagination.ts:41-45`, `apps/site/content/docs/v4/advanced/page-wrapping.mdx:52` | todo |  |
| Page total | page laid out again once the count is known | `packages/layout/src/steps/resolvePagination.ts:247-254` | todo |  |
| New engine | `experimentalPagination` or `layout` on any page switches the whole document | `packages/layout/src/index.ts:24-37` | todo |  |
| Page `layout` template | `(props: { children, pageNumber, totalPages, subPageNumber, subPageTotalPages }) => node`; must render children exactly once, or throws `[layout] A page layout must render its children exactly once (found n).`; no hooks; not in the docs site | `packages/renderer/index.d.ts:99-120`, `packages/layout/src/steps/resolvePageTemplates.ts:15-27` | todo |  |

### I. How the React renderer is built

- **Host config** (`packages/renderer/src/renderer.js:107-119`): `appendChild`,
  `appendChildToContainer`, `commitTextUpdate`, `commitUpdate`, `createInstance`,
  `createTextInstance`, `insertBefore`, `removeChild`, `removeChildFromContainer`, and
  `resetAfterCommit` wired to the `change` event. Instances are plain objects
  `{ type, box, style, props, children }` (`renderer.js:33-41`); the root container is
  `{ type: 'ROOT', document }` (`src/index.js:28`).
- **Three bundled reconcilers.** `react-reconciler` 0.23, 0.31 and 0.33 are aliased dev
  dependencies built into the package, and one is picked from `React.version` at load
  (`packages/reconciler/src/index.ts:6-20`, `packages/reconciler/package.json`). Props are compared shallowly
  before `commitUpdate` (`packages/reconciler/src/propsEqual.ts`).
- **Render props bypass the reconciler.** `render` and Page `layout` run during pagination, so
  they are wrapped to call `createInstances`, which calls function components directly
  (`renderer.js:7-31`, `packages/renderer/src/createInstances.js:46-48`). Hooks do not work there
  (`packages/renderer/index.d.ts:117-118`).
- **One renderer and one font store per process** (`packages/renderer/src/index.js:13-16`); the
  `@react-pdf/ui` notes say a registered family can never be removed on the main thread
  (`packages/ui/docs/web-workers.md`).
- **Pipeline entry** (`packages/renderer/src/index.js:38-95`): `new PDFDocument(...)` (pdfkit), then
  `layoutDocument(container.document, fontStore)`, then `renderPDF(ctx, layout)`. Layout is an
  `asyncCompose` of steps run bottom-up, from `resolveYoga` and `resolvePageSizes` through
  styles, inheritance, text layout, pagination, origins and `resolveZIndex`
  (`packages/layout/src/index.ts:39-59`). `@react-pdf/svgkit` renders the same layout to SVG.

### J. What the add-on packages give users

| Package | What a user writes | Source | Status | Fixture |
|---|---|---|---|---|
| `@react-pdf/tailwind` | `createTw(theme, { ptPerRem = 12 })` returns `tw('p-4 lg:p-8')` giving a `Style`; breakpoint and `landscape:` variants become media queries on the page; unknown classes warn once | `packages/tailwind/src/index.ts:46`, `apps/site/content/docs/v4/tailwind.mdx:43-90` | todo |  |
| `@react-pdf/math` | `<Math inline? width? height? color? debug?>{latex}</Math>`, drawn with the Svg primitives | `packages/math/src/Math.tsx:45-56`, `apps/site/content/docs/v4/addons/math.mdx` | todo |  |
| `@react-pdf/mermaid` | `<Mermaid width? height? color bg accent line muted surface>{graph}</Mermaid>`, also Svg primitives | `packages/mermaid/src/Mermaid.tsx`, `apps/site/content/docs/v4/addons/mermaid.mdx` | todo |  |
| `@react-pdf/hyphenate` | `Font.registerHyphenationCallback(syllables)` from `@react-pdf/hyphenate/de` and others | `apps/site/content/docs/v4/fonts.mdx:172-178` | todo |  |
| `@react-pdf/ui` | headless `Playground` (`Files`, `Editor`, `Document`, `Pagination`, `DownloadButton`) that transpiles code with sucrase and renders on the main thread | `packages/ui/README.md` | todo |  |

Math and Mermaid are plain components over the Svg primitives, so they work on axonpdf as soon as
those primitives do, with no engine work of their own.

### K. Where react-pdf's tests and examples are (fixture ideas only, never copied)

| What | Where | Notes |
|---|---|---|
| Visual snapshots | `packages/renderer/tests/*.test.jsx` (23 files), PNGs in `packages/renderer/tests/snapshots/` (96) | `renderToBuffer`, rasterised with `pdfjs-dist` and `@napi-rs/canvas`, compared with jest-image-snapshot (`packages/renderer/tests/renderComponent.js:1-43`) |
| Test images | `packages/renderer/tests/images/` (EXIF orientation 1 to 8, srcset) |  |
| Layout units | `packages/layout/tests/` (61 files: `steps/`, `node/`, `text/`, `paginate/`, `page/`), assets `packages/layout/tests/assets/` (font, png, jpg) | box positions after each step; best source for box-level parity |
| Text | `packages/textkit/tests/` (94 files) | line breaking, hyphenation, bidi |
| Styles | `packages/stylesheet/tests/` (15 files) | shorthand expansion and units |
| Other | `packages/render/tests/` (14), `packages/svgkit/tests/` (13, snapshots), `packages/svg/tests/`, `packages/font/tests/` (3), `packages/paginate/tests/` (2), `packages/tailwind/tests/`, `packages/math/tests/`, `packages/mermaid/tests/` |  |
| Example documents | `react-pdf/apps/examples/src/examples/` (32 folders), batch-rendered by `react-pdf/apps/examples/generate-pdfs.mjs` | resume, forms, svg, float, tailwind, emoji, media queries |
| Docs examples | `react-pdf/apps/site/lib/examples/` (44 files) and `apps/site/content/docs/v4/**/*.usage.jsx` | one per documented feature |
