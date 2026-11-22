# @axonpdf/core

The axonpdf engine compiled to WebAssembly, with a small JavaScript wrapper. It runs in Node,
browsers and Web Workers. It takes a document tree as JSON plus font and image bytes, and writes
the PDF in chunks as pages finish.

Most apps don't use this package directly. `@axonpdf/react` builds the tree from your components
and drives the engine for you. Use `@axonpdf/core` when you write a binding for another framework.

## Install

```sh
npm install @axonpdf/core
```

The packages aren't on npm yet. Until they are, build them from this repo (see the root README).

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

Every export is listed in [REFERENCE.md](REFERENCE.md). The project's [README](../../README.md)
covers the other packages and how to build them.
