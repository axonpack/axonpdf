import { describe, expect, it } from "vitest";

import { pdf } from "../pdf.js";
import { Document, Page } from "../primitives.js";
import { serializeDocument } from "../tree/serialize.js";
import { createSession } from "../tree/session.js";

describe("encryption props", () => {
  it("passes passwords and permissions and adds a 32-byte random seed", () => {
    const session = createSession();
    const document = (
      <Document
        userPassword="u"
        ownerPassword="o"
        permissions={{ printing: "lowResolution", copying: false }}
      >
        <Page />
      </Document>
    );
    const { props } = serializeDocument(pdf(document).container.document!, session);
    expect(props).toMatchObject({
      userPassword: "u",
      ownerPassword: "o",
      permissions: { printing: "lowResolution", copying: false },
    });
    const seed = props.encryptionSeed as number[];
    expect(seed).toHaveLength(32);
    expect(seed.every((b) => Number.isInteger(b) && b >= 0 && b < 256)).toBe(true);
  });
});
