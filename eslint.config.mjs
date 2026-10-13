// Flat ESLint config shared by every workspace. Each package's `lint` script runs
// `eslint src` from its own directory, and ESLint finds this file by walking up.
import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default tseslint.config(
    {
        ignores: [
            "**/dist/**",
            "**/node_modules/**",
            "**/generated/**",
            "**/target/**",
            "artifacts/**",
            "test/**",
        ],
    },
    js.configs.recommended,
    ...tseslint.configs.recommended,
    {
        // Existing backlog, tracked as warnings so the lint gate is green and the count is visible
        // (Z209 baseline; cleanup is Z151). New code should not add to these.
        rules: {
            "@typescript-eslint/no-explicit-any": "warn",
            "@typescript-eslint/no-unnecessary-type-constraint": "warn",
            "@typescript-eslint/no-unused-vars": [
                "warn",
                { argsIgnorePattern: "^_", varsIgnorePattern: "^_", caughtErrorsIgnorePattern: "^_" },
            ],
        },
    },
);
