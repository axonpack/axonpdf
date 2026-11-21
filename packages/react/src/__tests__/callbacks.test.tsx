import { afterEach, describe, expect, it } from "vitest";

import { Canvas, Document, Font, Page, Text, pdf } from "../index.node.js";

const LOGO =
  "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAIAAAAlC+aJAAAAv0lEQVR42u3a2Q2AMAwE0e2EPmiG/quAAriSEIn1aiwKmPdJbO3FRwAAAJg2y7q1f0aAru7pEv2YPoWh39M/MuRTP2aQT/oYQ4b1XQZ51rcbZFvfaJBzfYtB5vWvhmiASf2zQSXqHwyhAMP6O0MiwLb+0gDADWBefzYAAAAAAAAAAAAAAOAD4IcGwHcAzyoGAJ4WDQDlX6dZcHgAyu/IEraUCXvihE19wq1EwrVKyL1QwsVWyM1cyNUih68ASs8BKYYbcvdDOgQAAAAASUVORK5CYII=";

afterEach(() => {
  Font.registerHyphenationCallback(null as any);
  Font.registerEmojiSource(null as any);
});

describe("engine callbacks", () => {
  it("runs paint, hyphenation and emoji through the engine", async () => {
    const sizes: unknown[] = [];
    const words: string[] = [];
    const emoji: string[] = [];
    Font.registerHyphenationCallback((word) => [word]);
    Font.registerEmojiSource({
      builder: (code) => {
        emoji.push(code);
        return LOGO;
      },
    });
    const blob = await pdf(
      <Document>
        <Page>
          <Text
            hyphenationCallback={(word, builtin) => {
              words.push(word);
              return builtin(word);
            }}
          >
            Hyphenation everywhere 👍
          </Text>
          <Canvas
            style={{ width: 100, height: 50 }}
            paint={(painter, width, height) => {
              sizes.push([width, height]);
              painter.rect(0, 0, 10, 10).fill("red");
              return null;
            }}
          />
        </Page>
      </Document>,
    ).toBlob();

    expect(new TextDecoder().decode((await blob.arrayBuffer()).slice(0, 5))).toBe("%PDF-");
    expect(words).toContain("Hyphenation");
    expect(sizes).toContainEqual([100, 50]);
    expect(emoji).toEqual(["1f44d"]);
  });
});
