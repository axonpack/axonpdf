# @axonpdf/types

The TypeScript types a document's code names: styles, page sizes, image sources, bookmarks,
permissions and font descriptors, plus the `Primitive` enum of node types. `@axonpdf/react` and
`@axonpdf/stylesheet` take these types from here, and `@axonpdf/react` exports them again, so a
React app can import them from either package.

## Install

```sh
npm install @axonpdf/types
```

The packages aren't on npm yet. Until they are, build them from this repo (see the root README).

## Example

```ts
import type { PageSize, Style } from "@axonpdf/types";

const card: Style = {
  padding: "10 20",
  border: "1pt solid #ccc",
  "@media max-width: 400": { padding: 6 },
};
const size: PageSize = [300, "5in"];
```

Every export is listed in [REFERENCE.md](REFERENCE.md). The project's [README](../../README.md)
covers the other packages.
