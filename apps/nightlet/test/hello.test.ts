import { describe, expect, it } from "vitest";

import { hello } from "../src/hello.ts";

describe("hello", () => {
  it("returns a greeting", () => {
    expect(hello()).toBe("Hello, World!");
  });
});
