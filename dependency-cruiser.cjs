/** @type {import("dependency-cruiser").IConfiguration} */
module.exports = {
    forbidden: [
        {
            name: "no-circular",
            severity: "warn",
            comment: "Keep TypeScript workspace imports acyclic where possible.",
            from: {},
            to: { circular: true },
        },
        {
            name: "no-packages-to-apps",
            severity: "error",
            comment: "Shared packages must not import from app code.",
            from: { path: "^packages/[^/]+/src" },
            to: { path: "^apps/" },
        },
        {
            name: "no-foundation-to-ui-facing",
            severity: "error",
            comment: "Lower-level shared packages must not depend on engine, api, ui, or app layers.",
            from: { path: "^packages/(domain|content|core|commands|schemas|theme)/src" },
            to: { path: "^packages/(engine|api|ui)/src|^apps/" },
        },
        {
            name: "no-engine-or-api-to-ui",
            severity: "error",
            comment: "Engine and API packages must stay below the UI layer.",
            from: { path: "^packages/(engine|api)/src" },
            to: { path: "^packages/ui/src|^apps/" },
        },
        {
            name: "no-ui-to-apps",
            severity: "error",
            comment: "UI packages should stay reusable and not import from the web app.",
            from: { path: "^packages/ui/src" },
            to: { path: "^apps/" },
        },
    ],
    options: {
        doNotFollow: {
            path: "(^|/)(node_modules|dist|target|artifacts)(/|$)",
        },
        exclude: {
            path: "(^|/)(node_modules|dist|target|artifacts|playwright-report)(/|$)|\\.tsbuildinfo$",
        },
        tsPreCompilationDeps: true,
        combinedDependencies: true,
        enhancedResolveOptions: {
            extensions: [".ts", ".tsx", ".js", ".mjs", ".json"],
        },
        reporterOptions: {
            dot: {
                collapsePattern: "node_modules/[^/]+",
            },
        },
    },
};
