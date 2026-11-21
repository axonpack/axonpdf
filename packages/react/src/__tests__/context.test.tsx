import { Component, createContext, forwardRef, memo, useContext } from "react";
import { describe, expect, it } from "vitest";

import { Rows } from "../extras.js";
import { Document, Page, Text, View } from "../primitives.js";
import { reconciled, texts, walked } from "./walk.fixture.js";

const Theme = createContext("default");

const UseTheme = () => <Text>{`hook ${useContext(Theme)}`}</Text>;
const Memo = memo(() => <Text>{`memo ${useContext(Theme)}`}</Text>);
const Ref = forwardRef<unknown, { label: string }>(({ label }, ref) => (
  <Text>{`${label} ${ref === null ? "no ref" : "ref"} ${useContext(Theme)}`}</Text>
));

class ClassTheme extends Component {
  static contextType = Theme;
  render() {
    return <Text>{`class ${this.context}`}</Text>;
  }
}

const doc = (
  <Document>
    <Page>
      <UseTheme />
      <Theme.Provider value="outer">
        <UseTheme />
        <Theme.Consumer>{(value) => <Text>{`consumer ${value}`}</Text>}</Theme.Consumer>
        <View>
          <Theme.Provider value="inner">
            <Memo />
            <Ref label="forwarded" />
            <ClassTheme />
          </Theme.Provider>
        </View>
        <Rows rows={[1]}>{() => <UseTheme />}</Rows>
      </Theme.Provider>
    </Page>
  </Document>
);

describe("context in the one-shot render", () => {
  it("reads providers, consumers, class contextType and the default", () => {
    expect(texts(walked(doc))).toEqual([
      "hook default",
      "hook outer",
      "consumer outer",
      "memo inner",
      "forwarded no ref inner",
      "class inner",
    ]);
  });

  it("builds the same tree as the reconciler", () => {
    expect(walked(doc)).toBe(reconciled(doc));
  });
});
