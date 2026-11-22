# @axonpdf/stylesheet reference

## Exports

| Export | Kind |
| --- | --- |
| `default` | `resolveStyles(container, style)` |
| `flatten(style)` | function |
| `transformColor(value)` | function |
| `Container`, `SafeStyle` | types |
| `Style`, `StyleProp`, `Styles`, `Transform` | types, from `@axonpdf/types` |

## resolveStyles

```ts
resolveStyles(container: Container, style: StyleProp | null | undefined): SafeStyle
```

Runs these steps in order:

1. `flatten` merges a style list.
2. Each `@media` key whose query matches the container is merged in its place; the others are
   dropped.
3. Each key is expanded, in the order written, so a later key wins over an earlier one.

`undefined` values are skipped.

### Container

| Field | Meaning |
| --- | --- |
| `width`, `height` | The box the style resolves against, usually the page, in points. |
| `dpi` | Dots per inch for `px`. Default 72. |
| `remBase` | Points per `rem`. Default 18. |
| `orientation` | `landscape` or `portrait`. Left out, it is landscape when `width` is greater than `height`. |

### Lengths

Numbers are points. Strings with these units become points:

| Unit | Points |
| --- | --- |
| `pt` or none | the number |
| `in` | × 72 |
| `mm` | × 72 / 25.4 |
| `cm` | × 72 / 2.54 |
| `px` | × 72 / `dpi` |
| `rem` | × `remBase` |
| `vw` | % of the container's width |
| `vh` | % of the container's height |

Percentages, `auto` and other words stay as written.

### Shorthands

| Key | Becomes |
| --- | --- |
| `margin`, `padding` | One to four values, in CSS order, as `marginTop`, `marginRight`, `marginBottom`, `marginLeft` (and the same for `padding`). |
| `marginHorizontal`, `marginVertical`, `paddingHorizontal`, `paddingVertical` | The two sides on that axis. |
| `border`, `borderTop`, `borderRight`, `borderBottom`, `borderLeft` | `"<width> <style> <color>"` in any order, as `border<Side>Width`, `border<Side>Style` and `border<Side>Color`. Styles are `solid`, `dashed` and `dotted`. |
| `borderWidth`, `borderStyle`, `borderColor` | One to four values, as the four sides. |
| `borderRadius` | One to four values, as `borderTopLeftRadius`, `borderTopRightRadius`, `borderBottomRightRadius`, `borderBottomLeftRadius`. |
| `flex` | A number is `flexGrow`. `"<grow> <shrink> <basis>"` sets all three. |
| `flexFlow` | `flexDirection` and `flexWrap`. |
| `gap` | `rowGap` and `columnGap` (one value sets both). |
| `objectPosition` | `objectPositionX` and `objectPositionY`. Percentages become fractions (`50%` is 0.5); other words stay as written. |
| `transformOrigin` | `transformOriginX` and `transformOriginY`. `left`, `center`, `right`, `top` and `bottom` become percentages; the second value defaults to `50%`. |
| `transform`, `gradientTransform` | A string becomes a list of `{ operation, value }` (see below). A list stays as it is. |
| `fontWeight` | A weight name becomes its number (`bold` is 700). |
| any key ending in `color` or `Color`, `fill`, `stroke` | The color through `transformColor`. |

Every other key keeps its value, with a length turned into points.

### Transforms

`translate`, `translateX`, `translateY`, `scale`, `scaleX`, `scaleY`, `rotate`, `skew`, `skewX`,
`skewY` and `matrix`. Single-axis forms become the two-axis operation (`translateX(10)` is
`translate` with `[10, 0]`). Angles are degrees, or radians with `rad`. Translations take lengths
with units. Any other function throws `Unknown transform: <name>`.

### Media queries

A key such as `"@media max-width: 600"` holds a style that applies when its query matches the
container. Conditions are `min-width`, `max-width`, `min-height`, `max-height` (numbers in points)
and `orientation` (`landscape` or `portrait`), joined with `and`. Parentheses are optional.

## flatten

```ts
flatten(style: Style | Style[] | null | undefined | false): Style
```

Merges a style or a nested list of styles, left to right. `null`, `undefined` and `false` entries
are skipped.

## transformColor

```ts
transformColor(value: string): string
```

Turns `rgb()`, `rgba()`, `hsl()` and `hsla()` into hex: `#RRGGBB`, or `#RRGGBBAA` with an alpha.
Color names and hex colors are returned as written.
