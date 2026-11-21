import { Font } from "../font/font-store.js";
import type { Session } from "../tree/session.js";
import type { TreeDocument } from "../types/tree.type.js";
import { imageBytes } from "./image-bytes.js";

const VARIATION_SELECTOR = "\u{fe0f}";

/** The file name an emoji source uses: code points in hex, joined with `-`. */
const codePoints = (emoji: string, withVariationSelectors = false) =>
  Array.from(emoji)
    .filter((char) => withVariationSelectors || char !== VARIATION_SELECTOR)
    .map((char) => char.codePointAt(0)!.toString(16))
    .join("-");

/**
 * Fetches an image for every emoji the text holds from the registered emoji source, and lists
 * them in the document's `emoji` prop. An emoji whose image fails to load stays text.
 */
export async function addEmoji(tree: TreeDocument, session: Session) {
  const source = Font.getEmojiSource();
  if (!source || !session.emoji.size) return;
  const emoji: Record<string, { asset: number }> = {};
  await Promise.all(
    [...session.emoji].map(async (char) => {
      const code = codePoints(char, source.withVariationSelectors);
      const url =
        "builder" in source
          ? source.builder(code)
          : `${source.url}${code}.${source.format ?? "png"}`;
      try {
        const bytes = await imageBytes(url);
        const id = session.nextId++;
        session.assets.set(id, bytes);
        emoji[char] = { asset: id };
      } catch (error) {
        console.warn(error, "Failed to load emoji image");
      }
    }),
  );
  tree.props.emoji = emoji;
}
