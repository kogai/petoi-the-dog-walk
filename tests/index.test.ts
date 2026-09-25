import { expect, test } from "vitest";
import { version } from "../src/index.ts";

test("package exposes its version", () => {
  expect(version).toBe("0.0.0");
});
