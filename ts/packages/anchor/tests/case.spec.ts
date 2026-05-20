import { toCamelCase } from "../src/utils/case";

// The npm `camelcase` library treats a digit followed by a letter as a word
// boundary (e.g. `a1bReceive` -> `a1BReceive`), but the Rust-side IDL
// generator uses `heck::to_lower_camel_case`, which does not split on
// digit-letter transitions. The mismatch caused TS-typed lookups for names
// like `a1bReceive` / `myA1bParam` to miss their runtime entries — which
// in the instruction-arg case silently defaulted to the zero pubkey.
// `toCamelCase` matches the Rust-side semantics.
describe("toCamelCase", () => {
  test("converts snake_case to camelCase", () => {
    expect(toCamelCase("foo_bar")).toBe("fooBar");
    expect(toCamelCase("set_pass_threshold_bps")).toBe("setPassThresholdBps");
  });

  test("is idempotent on camelCase input", () => {
    expect(toCamelCase("fooBar")).toBe("fooBar");
    expect(toCamelCase("setPassThresholdBps")).toBe("setPassThresholdBps");
  });

  test("does not split on digit-letter transitions (#3043)", () => {
    expect(toCamelCase("a1b_receive")).toBe("a1bReceive");
    expect(toCamelCase("a1bReceive")).toBe("a1bReceive");
    expect(toCamelCase("my_a1b_param")).toBe("myA1bParam");
    expect(toCamelCase("myA1bParam")).toBe("myA1bParam");
  });

  test("handles leading and trailing digits", () => {
    expect(toCamelCase("foo123")).toBe("foo123");
    expect(toCamelCase("foo_123")).toBe("foo123");
    expect(toCamelCase("123_foo")).toBe("123Foo");
  });

  test("handles acronyms as a single word", () => {
    // `ABCFoo` -> [`ABC`, `Foo`] -> `abcFoo`
    expect(toCamelCase("ABCFoo")).toBe("abcFoo");
    expect(toCamelCase("ABC_foo")).toBe("abcFoo");
  });

  test("accepts hyphen / space / dot separators", () => {
    expect(toCamelCase("foo-bar")).toBe("fooBar");
    expect(toCamelCase("foo bar")).toBe("fooBar");
    expect(toCamelCase("foo.bar")).toBe("fooBar");
  });

  test("handles single word and empty input", () => {
    expect(toCamelCase("initialize")).toBe("initialize");
    expect(toCamelCase("")).toBe("");
  });
});
