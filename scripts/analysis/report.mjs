import path from "node:path";
import {
    ANALYSIS_DIR,
    advisoryExitCode,
    ensureDir,
    readJsonIfExists,
    writeJson,
    writeText,
    printSection,
} from "./common.mjs";

const OUTPUT_DIR = ANALYSIS_DIR;

async function main() {
    await ensureDir(OUTPUT_DIR);

    const rustCoupling = await readJsonIfExists(path.join(OUTPUT_DIR, "rust-coupling", "summary.json"));
    const rustMetrics = await readJsonIfExists(path.join(OUTPUT_DIR, "rust-metrics", "summary.json"));
    const tsDeps = await readJsonIfExists(path.join(OUTPUT_DIR, "ts-deps", "summary.json"));

    const summary = {
        generatedAt: new Date().toISOString(),
        advisory: true,
        rustCoupling: rustCoupling
            ? {
                  violationCount: rustCoupling.violationCount,
                  cycleCount: rustCoupling.cycleCount,
                  externalToolInstalled: rustCoupling.externalTool?.installed ?? false,
              }
            : null,
        rustMetrics: rustMetrics
            ? {
                  fileCount: rustMetrics.totals?.fileCount ?? 0,
                  loc: rustMetrics.totals?.loc ?? 0,
                  functionCount: rustMetrics.totals?.functionCount ?? 0,
                  externalToolInstalled: rustMetrics.externalTool?.installed ?? false,
              }
            : null,
        tsDeps: tsDeps
            ? {
                  installed: tsDeps.installed,
                  moduleCount: tsDeps.moduleCount,
                  violationCount: tsDeps.violationCount,
              }
            : null,
    };

    const markdown = [
        "# Analysis Summary",
        "",
        `Generated: ${summary.generatedAt}`,
        "",
        "## Rust coupling",
        rustCoupling
            ? `Violations: ${rustCoupling.violationCount}, cycles: ${rustCoupling.cycleCount}, cargo-coupling installed: ${rustCoupling.externalTool?.installed ? "yes" : "no"}`
            : "No report found.",
        "",
        "## Rust metrics",
        rustMetrics
            ? `Files: ${rustMetrics.totals?.fileCount ?? 0}, LOC: ${rustMetrics.totals?.loc ?? 0}, functions: ${rustMetrics.totals?.functionCount ?? 0}, rust-code-analysis installed: ${rustMetrics.externalTool?.installed ? "yes" : "no"}`
            : "No report found.",
        "",
        "## TypeScript dependencies",
        tsDeps
            ? `dependency-cruiser installed: ${tsDeps.installed ? "yes" : "no"}, modules: ${tsDeps.moduleCount ?? "n/a"}, violations: ${tsDeps.violationCount ?? "n/a"}`
            : "No report found.",
        "",
        "All analysis commands remain advisory in this phase.",
        "",
    ].join("\n");

    await writeJson(path.join(OUTPUT_DIR, "summary.json"), summary);
    await writeText(path.join(OUTPUT_DIR, "summary.md"), `${markdown}\n`);

    printSection("Analysis Report");
    console.log(markdown);
    advisoryExitCode();
}

main().catch((error) => {
    console.error(error instanceof Error ? error.stack || error.message : String(error));
    process.exitCode = 1;
});
