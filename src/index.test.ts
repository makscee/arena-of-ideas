import { expect, test } from "vitest";
import { KERNEL_VERSION } from "./index.js";

test("scaffold is alive", () => {
  expect(KERNEL_VERSION).toBe("5.0.0-alpha.0");
});
