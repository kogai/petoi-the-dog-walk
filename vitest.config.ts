// Test policy: docs/rules/testing.md
// *.jev.test.ts and *.hardware.test.ts only run through `pnpm test:jev` / `pnpm test:hardware`.
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: "default",
          include: ["tests/**/*.test.ts"],
          exclude: ["tests/**/*.jev.test.ts", "tests/**/*.hardware.test.ts"],
        },
      },
      { test: { name: "jev", include: ["tests/**/*.jev.test.ts"] } },
      { test: { name: "hardware", include: ["tests/**/*.hardware.test.ts"] } },
    ],
    coverage: {
      provider: "v8",
      include: ["src/**/*.ts"],
      thresholds: { branches: 80, lines: 80, functions: 80, statements: 80 },
    },
  },
});
