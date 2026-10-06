import { describe, expect, it } from "vitest";

import { app } from "../src/app.ts";

describe("GET /health", () => {
  it("returns OK", async () => {
    const res = await app.request("/health");

    expect(res.status).toBe(200);
    await expect(res.text()).resolves.toBe("OK");
  });
});
