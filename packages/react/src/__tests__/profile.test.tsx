import { describe, expect, it } from "vitest";

import { Document, Page, Text, View, formatProfile, profile } from "../index.node.js";

const doc = (
  <Document>
    <Page size="A6">
      {Array.from({ length: 60 }, (_, i) => (
        <View key={i} wrap={false} style={{ borderBottomWidth: 1, padding: 4 }}>
          <Text>{`Line ${i}: some words to lay out and break.`}</Text>
        </View>
      ))}
      <Text fixed render={({ pageNumber }) => `Page ${pageNumber}`} />
    </Page>
  </Document>
);

describe("profile", () => {
  it("times every phase, and the phases add up to the total", async () => {
    const report = await profile(doc);
    const { phases, engine, pages, cpu } = report;
    const sum = Object.values(phases).reduce((a, b) => a + b, 0);
    expect(Math.abs(sum - report.totalMs)).toBeLessThan(1);
    expect(phases.react).toBeGreaterThan(0);
    expect(engine.layout + engine.text + engine.write).toBeGreaterThan(0);
    const inside = Object.values(engine).reduce((a, b) => a + b, 0);
    expect(Math.abs(inside - phases.engine)).toBeLessThan(1);
    expect(pages.length).toBeGreaterThan(1);
    expect(report.bytes).toBeGreaterThan(1000);
    expect(report.memory.wasm).toBeGreaterThan(0);
    expect(cpu?.groups.some((g) => g.name === "axonpdf")).toBe(true);
    expect(formatProfile(report)).toContain("React render");
  });
});

const spin = (ms: number) => {
  const end = performance.now() + ms;
  while (performance.now() < end);
};

function Slow({ i }: { i: number }) {
  spin(1);
  return <Text>{`Row ${i}`}</Text>;
}

describe("profile with lazy content", () => {
  it("counts components as React and render props as the engine's callbacks", async () => {
    const report = await profile(
      <Document>
        <Page size="A6">
          <View>
            {Array.from({ length: 150 }, (_, i) => (
              <Slow key={i} i={i} />
            ))}
          </View>
          <Text
            fixed
            render={({ pageNumber }) => {
              spin(2);
              return `Page ${pageNumber}`;
            }}
          />
        </Page>
      </Document>,
    );
    const { phases, engine } = report;
    // 150 components of 1 ms, most of them built while the engine runs.
    expect(phases.react).toBeGreaterThan(140);
    expect(engine.callbacks).toBeGreaterThan(2);
    expect(engine.callbacks).toBeLessThan(phases.react);
    const sum = Object.values(phases).reduce((a, b) => a + b, 0);
    expect(Math.abs(sum - report.totalMs)).toBeLessThan(1);
    const inside = Object.values(engine).reduce((a, b) => a + b, 0);
    expect(Math.abs(inside - phases.engine)).toBeLessThan(1);
  });
});
