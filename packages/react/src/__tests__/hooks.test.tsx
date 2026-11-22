import {
  Component,
  useCallback,
  useEffect,
  useId,
  useMemo,
  useReducer,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import { describe, expect, it } from "vitest";

import { Document, Page, Text } from "../primitives.js";
import { reconciled, texts, walked } from "./walk.fixture.js";

function Hooks() {
  const [count] = useState(() => 2);
  const [total] = useReducer(
    (s: number) => s + 1,
    10,
    (n: number) => n * 3,
  );
  const doubled = useMemo(() => count * 2, [count]);
  const format = useCallback((n: number) => `#${n}`, []);
  const ref = useRef("ref");
  const store = useSyncExternalStore(
    () => () => {},
    () => "store",
  );
  const id = useId();
  const [effect, setEffect] = useState("before");
  // oxlint-disable-next-line react/set-state-in-effect -- shows effects are not run
  useEffect(() => setEffect("after"), []);
  return (
    <Text>
      {/* oxlint-disable-next-line react/refs -- a first-render value */}
      {[count, total, doubled, format(1), ref.current, store, typeof id, effect].join(" ")}
    </Text>
  );
}

class Counter extends Component<{ start: number }, { n: number }> {
  static defaultProps = { start: 5 };
  state = { n: this.props.start };
  static getDerivedStateFromProps(props: { start: number }, state: { n: number }) {
    return { n: state.n + props.start };
  }
  render() {
    return <Text>{`class ${this.state.n}`}</Text>;
  }
}

class WillMount extends Component<object, { ready: boolean }> {
  state = { ready: false };
  // oxlint-disable-next-line react/no-unsafe -- still part of React's class API
  UNSAFE_componentWillMount() {
    this.setState({ ready: true });
  }
  render() {
    return <Text>{this.state.ready ? "ready" : "not ready"}</Text>;
  }
}

const doc = (
  <Document>
    <Page>
      <Hooks />
      <Counter />
      <WillMount />
    </Page>
  </Document>
);

describe("hooks in the one-shot render", () => {
  it("gives first-render values, runs no effects, and mounts classes", () => {
    expect(texts(walked(doc))).toEqual(["2 30 4 #1 ref store string before", "class 10", "ready"]);
  });

  it("builds the same tree as the reconciler", () => {
    expect(walked(doc)).toBe(reconciled(doc));
  });
});
