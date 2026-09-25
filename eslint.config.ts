// Lint rules: docs/rules/static-analysis.md
import js from "@eslint/js";
import functional from "eslint-plugin-functional";
import { defineConfig } from "eslint/config";
import tseslint from "typescript-eslint";

export default defineConfig(
  { ignores: ["node_modules/", "coverage/", "*.cjs"] },
  js.configs.recommended,
  tseslint.configs.strictTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  {
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    rules: {
      "@typescript-eslint/switch-exhaustiveness-check": "error",
      "@typescript-eslint/explicit-module-boundary-types": "error",
      "@typescript-eslint/consistent-type-definitions": ["error", "type"],
      "@typescript-eslint/ban-ts-comment": [
        "error",
        { "ts-expect-error": "allow-with-description", minimumDescriptionLength: 10 },
      ],
      "no-console": "error",
    },
  },
  // Functional core: domain logic is pure and immutable (docs/rules/coding.md).
  {
    files: ["src/**/*.ts"],
    ignores: ["src/**/shell/**", "src/main.ts"],
    plugins: { functional },
    rules: {
      "functional/no-let": "error",
      "functional/immutable-data": "error",
      "functional/no-loop-statements": "error",
      "functional/no-classes": "error",
      "functional/no-this-expressions": "error",
      "functional/no-throw-statements": "error",
      "functional/no-try-statements": "error",
      "functional/prefer-readonly-type": "off",
      "functional/prefer-immutable-types": "off",
    },
  },
  {
    files: ["tests/**/*.ts", "scripts/**/*.ts"],
    rules: { "no-console": "off" },
  },
);
