import { describe, expect, it, vi } from "vitest";

import FontStore, { sharedFontStore } from "../index.js";

describe("FontStore", () => {
  it("starts with the standard families and registers single and bulk fonts", () => {
    const store = new FontStore();
    expect(store.getRegisteredFontFamilies()).toEqual(["Helvetica", "Courier", "Times-Roman"]);
    store.register({ family: "Inter", src: "inter.ttf", fontWeight: "semibold" });
    store.register({
      family: "Inter",
      fonts: [
        { src: "inter-italic.ttf", fontStyle: "italic" },
        { src: "inter-black.ttf", fontWeight: 900 },
      ],
    });
    expect(store.getFont({ fontFamily: "Inter", fontWeight: 600 })?.src).toBe("inter.ttf");
    expect(store.getFont({ fontFamily: "Inter", fontWeight: "bold" })?.src).toBe("inter.ttf");
    expect(store.getFont({ fontFamily: "Inter", fontWeight: 800 })?.src).toBe("inter-black.ttf");
    expect(store.getFont({ fontFamily: "Inter", fontStyle: "italic" })?.src).toBe(
      "inter-italic.ttf",
    );
    expect(() => store.getFont({ fontFamily: "Nope" })).toThrow(
      "Font family not registered: Nope. Please register it calling Font.register() method.",
    );
  });

  it("keeps the emoji source and hyphenation callback until cleared", () => {
    const store = new FontStore();
    const callback = (word: string) => [word];
    store.registerEmojiSource({ url: "https://emoji/", format: "png" });
    store.registerHyphenationCallback(callback);
    expect(store.getEmojiSource()).toEqual({ url: "https://emoji/", format: "png" });
    expect(store.getHyphenationCallback()).toBe(callback);
    store.clear();
    expect(store.getRegisteredFontFamilies()).toEqual([]);
    expect(store.getEmojiSource()).toBeNull();
    expect(store.getHyphenationCallback()).toBeNull();
  });

  it("loads a font's bytes once, and forgets them on reset", async () => {
    const store = new FontStore();
    store.loader = vi.fn(async () => new Uint8Array([1, 2, 3]));
    store.register({ family: "Inter", src: "https://fonts/inter.ttf", method: "GET" });
    await store.load({ fontFamily: "Inter" });
    await store.load({ fontFamily: "Inter" });
    expect(store.loader).toHaveBeenCalledTimes(1);
    expect(store.loader).toHaveBeenCalledWith("https://fonts/inter.ttf", { method: "GET" });
    expect(store.getFont({ fontFamily: "Inter" })?.data).toEqual(new Uint8Array([1, 2, 3]));
    store.reset();
    expect(store.getFont({ fontFamily: "Inter" })?.data).toBeNull();
  });

  it("shares one store between every copy of the package", async () => {
    vi.resetModules();
    const copy = await import("../index.js?copy" as string);
    expect(copy.sharedFontStore).toBe(sharedFontStore);
    expect(copy.default).not.toBe(FontStore);
  });
});
