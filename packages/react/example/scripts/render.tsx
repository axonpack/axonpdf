import { fileURLToPath } from "node:url";

import { renderToFile } from "@axonpdf/react";

import { Invoice } from "../src/invoice.js";

const logo = fileURLToPath(new URL("../public/logo.png", import.meta.url));
const out = fileURLToPath(new URL("../out.pdf", import.meta.url));

await renderToFile(<Invoice logo={logo} />, out);
console.log(`Wrote ${out}`);
