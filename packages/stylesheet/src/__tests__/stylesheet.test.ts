import { describe, expect, it } from "vitest";

import stylesheet, { flatten, transformColor, type Style } from "../index.js";

const container = { width: 400, height: 600, orientation: "portrait" as const };

describe("stylesheet", () => {
  it("resolves a style as documented", () => {
    expect(
      stylesheet(container, {
        margin: 20,
        width: "50vw",
        height: "20vh",
        borderRadius: 5,
        fontWeight: "semibold",
        borderBottom: "2 solid yellow",
        "@media max-width: 500": { backgroundColor: "rgb(255, 0, 0)" },
      }),
    ).toEqual({
      width: 200,
      height: 120,
      marginTop: 20,
      marginLeft: 20,
      marginRight: 20,
      marginBottom: 20,
      borderTopLeftRadius: 5,
      borderTopRightRadius: 5,
      borderBottomLeftRadius: 5,
      borderBottomRightRadius: 5,
      fontWeight: 600,
      borderBottomWidth: 2,
      borderBottomStyle: "solid",
      borderBottomColor: "yellow",
      backgroundColor: "#FF0000",
    });
  });

  it("flattens lists, skipping empty entries", () => {
    expect(
      flatten([{ margin: 10, padding: 5 }, null, [{ backgroundColor: "blue" }], undefined]),
    ).toEqual({
      margin: 10,
      padding: 5,
      backgroundColor: "blue",
    });
    expect(stylesheet(container, [{ padding: 5 }, false as never, { paddingTop: 1 }])).toEqual({
      paddingTop: 1,
      paddingRight: 5,
      paddingBottom: 5,
      paddingLeft: 5,
    });
  });

  it("applies matching media queries only", () => {
    const style = {
      fontSize: 12,
      "@media max-width: 500": { fontSize: 10 },
      "@media orientation: landscape": { flexDirection: "row" as const },
      "@media min-width: 400 and max-width: 800": { padding: 20 },
    };
    expect(stylesheet(container, style)).toMatchObject({ fontSize: 10, paddingTop: 20 });
    expect(stylesheet(container, style).flexDirection).toBeUndefined();
    expect(stylesheet({ width: 900, height: 600 }, style)).toEqual({
      fontSize: 12,
      flexDirection: "row",
    });
  });

  it("converts units to points", () => {
    const s = stylesheet(
      { ...container, dpi: 144 },
      {
        width: "2in",
        height: "50mm",
        top: "1cm",
        margin: "10vh",
        fontSize: "5vw",
        gap: "2rem",
        left: "100px",
        maxWidth: "50%",
      },
    );
    expect(s).toMatchObject({
      width: 144,
      top: 72 / 2.54,
      marginTop: 60,
      fontSize: 20,
      rowGap: 36,
      columnGap: 36,
      left: 50,
      maxWidth: "50%",
    });
    expect(s.height).toBeCloseTo(141.73, 2);
  });

  it("turns colors into hex", () => {
    expect(transformColor("rgb(255, 0, 0)")).toBe("#FF0000");
    expect(transformColor("rgba(0, 0, 255, 0.5)")).toBe("#0000FF80");
    expect(transformColor("hsl(120, 100%, 50%)")).toBe("#00FF00");
    expect(transformColor("hsla(0, 100%, 50%, 0.8)")).toBe("#FF0000CC");
    expect(transformColor("rgb(255, 128, 0)")).toBe("#FF8000");
    expect(transformColor("red")).toBe("red");
  });

  it("expands shorthands", () => {
    const r = (style: Style) => stylesheet(container, style);
    expect(r({ margin: "10 20 30 40" })).toEqual({
      marginTop: 10,
      marginRight: 20,
      marginBottom: 30,
      marginLeft: 40,
    });
    expect(r({ marginHorizontal: 20, marginVertical: 10 })).toEqual({
      marginTop: 10,
      marginRight: 20,
      marginBottom: 10,
      marginLeft: 20,
    });
    expect(r({ border: "2 solid red" })).toEqual(
      Object.fromEntries(
        ["Top", "Right", "Bottom", "Left"].flatMap((s) => [
          [`border${s}Width`, 2],
          [`border${s}Style`, "solid"],
          [`border${s}Color`, "red"],
        ]),
      ),
    );
    expect(r({ flex: "1 0 auto" })).toEqual({ flexGrow: 1, flexShrink: 0, flexBasis: "auto" });
    expect(r({ flex: 1 })).toEqual({ flexGrow: 1 });
    expect(r({ gap: "10 20" })).toEqual({ rowGap: 10, columnGap: 20 });
    expect(r({ objectPosition: "50% 25%" })).toEqual({
      objectPositionX: 0.5,
      objectPositionY: 0.25,
    });
    expect(r({ transformOrigin: "center top" })).toEqual({
      transformOriginX: "50%",
      transformOriginY: "0%",
    });
  });

  it("parses transforms", () => {
    expect(stylesheet(container, { transform: "translate(10, 20) rotate(45) scale(2)" })).toEqual({
      transform: [
        { operation: "translate", value: [10, 20] },
        { operation: "rotate", value: [45] },
        { operation: "scale", value: [2, 2] },
      ],
    });
    expect(
      stylesheet(container, { transform: "scaleX(2) skewY(10) matrix(1, 0, 0, 1, 5, 6)" })
        .transform,
    ).toEqual([
      { operation: "scale", value: [2, 1] },
      { operation: "skew", value: [0, 10] },
      { operation: "matrix", value: [1, 0, 0, 1, 5, 6] },
    ]);
  });
});
