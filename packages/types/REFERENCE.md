# @axonpdf/types reference

Everything here is a type, except `Primitive`.

## Styles

| Type | What it is |
| --- | --- |
| `BaseStyle` | Every style key the engine reads. The keys and their values are listed in the `@axonpdf/react` reference, under Styles. |
| `MediaQueryStyle` | `{ "@media <query>": BaseStyle }` |
| `Style` | `BaseStyle & MediaQueryStyle` |
| `StyleProp` | A `Style`, or a list of styles nested to any depth, with `undefined`, `null` and `false` entries allowed. |
| `Styles` | `{ [name: string]: Style }`, what `StyleSheet.create` takes. |
| `BorderStyleValue` | `"solid" \| "dashed" \| "dotted"` |
| `Transform` | One resolved transform: `{ operation: "scale" \| "translate" \| "rotate" \| "skew" \| "matrix", value: number[] }`. |
| `ShapeOutside` | A resolved `shapeOutside`: a circle, ellipse, polygon or inset. |
| `FontStyle` | `"normal" \| "italic" \| "oblique"` |
| `FontWeight` | A number or a weight name (`thin` to `black`). |
| `SVGPresentationAttributes` | What SVG shapes paint with: `fill`, `color`, `stroke`, `transform`, `strokeDasharray`, `opacity`, `strokeWidth`, `fillOpacity`, `fillRule`, `strokeOpacity`, `textAnchor`, `strokeLinecap`, `strokeLinejoin`, `visibility`, `clipPath`, `markerStart`, `markerMid`, `markerEnd`, `dominantBaseline`. |

## Pages and the document

| Type | Values |
| --- | --- |
| `PageSize` | A size name (`4A0`, `2A0`, `A0` to `A10`, `B0` to `B10`, `C0` to `C10`, `RA0` to `RA4`, `SRA0` to `SRA4`, `EXECUTIVE`, `FOLIO`, `LEGAL`, `LETTER`, `TABLOID`, `ID1`), `[width]`, `[width, height]`, or `{ width, height? }`. Lengths are numbers in points or strings with a unit. A size without a height grows with its content. |
| `Orientation` | `"portrait" \| "landscape"` |
| `PDFVersion` | `"1.3" \| "1.4" \| "1.5" \| "1.6" \| "1.7" \| "1.7ext3"` |
| `PDFConformance` | `"PDF/A-1" \| "PDF/A-1b" \| "PDF/A-2" \| "PDF/A-2b" \| "PDF/A-3" \| "PDF/A-3b"` |
| `PageLayout` | `"singlePage" \| "oneColumn" \| "twoColumnLeft" \| "twoColumnRight" \| "twoPageLeft" \| "twoPageRight"` |
| `PageMode` | `"useNone" \| "useOutlines" \| "useThumbs" \| "fullScreen" \| "useOC" \| "useAttachments"` |
| `Permissions` | `{ printing?: "lowResolution" \| "highResolution", modifying?, copying?, annotating?, fillingForms?, contentAccessibility?, documentAssembly? }`, all booleans but `printing`. |
| `Bookmark` | A title, or `{ title, top?, left?, zoom?, fit?, expanded? }`. |
| `HitSlop` | A number, or `{ top?, bottom?, left?, right? }`. |

## Images

| Type | Values |
| --- | --- |
| `SourceObject` | A source, a function that returns one, a promise of one, or a function that returns a promise of one. |
| (a source) | A URL or path string, a `Buffer`, `Uint8Array`, `ArrayBuffer` or `Blob`, `{ data, format: "png" \| "jpg" }`, or `{ uri, method?, headers?, body?, credentials? }`. |
| `SrcSet` | `string`, such as `"small.jpg 300w, large.jpg 900w"` |
| `Sizes` | `string \| number` |

## Fonts

From `@axonpdf/font`: `FontDescriptor`, `FontStore` (the store's type), `EmojiSource` and
`HyphenationCallback`.

## Primitive

An enum of the node types, with the same values as the components in `@axonpdf/react`:
`G`, `Svg`, `View`, `Text`, `Link`, `Page`, `Note`, `Path`, `Rect`, `Line`, `Stop`, `Defs`,
`Image`, `Tspan`, `Canvas`, `Circle`, `Ellipse`, `Polygon`, `Document`, `Polyline`, `ClipPath`,
`TextInstance`, `Fragment`, `LinearGradient` and `RadialGradient`. Each value is the type's name
in upper case with underscores (`Primitive.ClipPath` is `"CLIP_PATH"`).
