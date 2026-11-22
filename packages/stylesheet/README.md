# @axonpdf/stylesheet

Resolves a document style in JavaScript: style lists are merged, matching `@media` blocks are
applied, shorthands are expanded, lengths become points and colors become hex. Use it when your
own code needs to read a style's final values, for example to measure something before it is
rendered.

`@axonpdf/react` doesn't need this package. The engine resolves styles itself when it lays the
document out.

## Install

```sh
npm install @axonpdf/stylesheet
```

The packages aren't on npm yet. Until they are, build them from this repo (see the root README).

## Example

```ts
import resolveStyles from "@axonpdf/stylesheet";

const container = { width: 595.28, height: 841.89 };
resolveStyles(container, [
  { margin: "1in 20", border: "1pt solid red" },
  { "@media max-width: 600": { color: "rgb(255, 0, 0)" } },
]);
// { marginTop: 72, marginRight: 20, marginBottom: 72, marginLeft: 20,
//   borderTopWidth: 1, borderTopStyle: "solid", borderTopColor: "red", ...,
//   color: "#FF0000" }
```

Every export is listed in [REFERENCE.md](REFERENCE.md). The project's [README](../../README.md)
covers the other packages.
