import { defineConfig } from "vitest/config";

/** Shared Vitest config for the `packages/*` workspaces; coverage lands in `artifacts/coverage/<name>`. */
export function packageConfig(name: string, options: { dom?: boolean } = {}) {
    return defineConfig({
        test: {
            include: ["src/**/*.test.ts"],
            environment: options.dom ? "happy-dom" : "node",
            coverage: {
                provider: "v8",
                include: ["src/**/*.ts"],
                exclude: ["src/**/*.test.ts", "src/**/*.d.ts", "src/**/generated/**", "src/**/testing/**"],
                reporter: ["text-summary", "json-summary", "html"],
                reportsDirectory: `../../artifacts/coverage/${name}`,
            },
        },
    });
}
