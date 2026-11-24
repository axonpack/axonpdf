<div align="center">

# @axonpdf/core

**The axonpdf engine as WebAssembly, for Node, browsers and Web Workers.**

**[Documentation](https://axonpack.dev/docs/axonpdf/core/)** · [Reference](https://axonpack.dev/docs/axonpdf/core/reference/)

[![npm version](https://img.shields.io/npm/v/@axonpdf/core.svg)](https://www.npmjs.com/package/@axonpdf/core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/axonpack/axonpdf/blob/main/LICENSE)

</div>

The engine takes a document tree as JSON plus font and image bytes, lays it out, and writes the
PDF in chunks as pages finish. Most apps don't use this package directly:
[`@axonpdf/react`](https://axonpack.dev/docs/axonpdf/react/) builds the tree from your components
and drives the engine for you. Use `@axonpdf/core` to write a binding for another framework.

## Install

```sh
npm install @axonpdf/core
```

## Example

```ts
import { createEngine } from "@axonpdf/core";

const engine = await createEngine();
const chunks: Uint8Array[] = [];
const tree = {
  v: 1,
  type: "DOCUMENT",
  props: { title: "Hello" },
  children: [
    {
      type: "PAGE",
      props: { size: "A4" },
      style: { padding: 40 },
      children: [
        { type: "TEXT", props: {}, children: [{ type: "TEXT_INSTANCE", value: "Hello" }] },
      ],
    },
  ],
};
engine.render(JSON.stringify(tree), (chunk) => chunks.push(chunk));
engine.free();
```

Every export, engine method, callback and error is in the
[reference](https://axonpack.dev/docs/axonpdf/core/reference/) and in [REFERENCE.md](REFERENCE.md).

## License

MIT
