import type { Styles } from "./types/style.type.js";

export const StyleSheet = {
  create: <T extends Styles>(styles: T): T => styles,
};
