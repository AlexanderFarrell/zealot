import path from "node:path";
import fsp from "node:fs/promises";
import {
    ANALYSIS_DIR,
    advisoryExitCode,
    ensureDir,
    fileExists,
    printSection,
    runCommand,
    writeJson,
    writeText,
} from "./common.mjs";

const OUTPUT_DIR = path.join(ANALYSIS_DIR, "ts-deps");
const LOCAL_DEPCRUISE = path.join(process.cwd(), "node_modules", ".bin", "depcruise");

async function main() {
    await ensureDir(OUTPUT_DIR);

    const scanPaths = await collectScanPaths();
    const configFile = "dependency-cruiser.cjs";

    const jsonFile = path.join(OUTPUT_DIR, "dependency-cruiser-report.json");
    const textFile = path.join(OUTPUT_DIR, "dependency-cruiser-report.txt");
    const dotFile = path.join(OUTPUT_DIR, "dependency-cruiser-graph.dot");
    const summaryFile = path.join(OUTPUT_DIR, "summary.json");
    const noteFile = path.join(OUTPUT_DIR, "summary.txt");

    if (!fileExists(LOCAL_DEPCRUISE)) {
        const installHint = "dependency-cruiser is not installed. Install it with: npm install --save-dev dependency-cruiser";
        const summary = {
            generatedAt: new Date().toISOString(),
            advisory: true,
            installed: false,
            scanPaths,
            installHint,
        };

        await writeJson(summaryFile, summary);
        await writeText(noteFile, `${installHint}\n`);

        printSection("TypeScript Dependencies");
        console.log(installHint);
        advisoryExitCode();
        return;
    }

    const baseArgs = ["--config", configFile, ...scanPaths];
    const jsonResult = runCommand(LOCAL_DEPCRUISE, [...baseArgs, "--output-type", "json"], { allowFailure: true });
    await writeText(jsonFile, jsonResult.stdout || "");

    const textResult = runCommand(LOCAL_DEPCRUISE, [...baseArgs, "--output-type", "err-long"], { allowFailure: true });
    await writeText(textFile, `${textResult.stdout || ""}${textResult.stderr || ""}`);

    const dotResult = runCommand(LOCAL_DEPCRUISE, [...baseArgs, "--output-type", "dot"], { allowFailure: true });
    await writeText(dotFile, dotResult.stdout || "");

    let report = null;
    try {
        report = JSON.parse(jsonResult.stdout || "null");
    } catch (_error) {
        report = null;
    }

    const summary = {
        generatedAt: new Date().toISOString(),
        advisory: true,
        installed: true,
        command: jsonResult.command,
        scanPaths,
        exitCodes: {
            json: jsonResult.exitCode,
            text: textResult.exitCode,
            dot: dotResult.exitCode,
        },
        moduleCount: Array.isArray(report?.modules) ? report.modules.length : null,
        violationCount: Array.isArray(report?.summary?.violations) ? report.summary.violations.length : null,
        outputFiles: {
            jsonFile,
            textFile,
            dotFile,
        },
    };

    const lines = [
        "TypeScript dependency analysis",
        "",
        `Scanned roots: ${scanPaths.join(", ")}`,
        `JSON exit code: ${jsonResult.exitCode}`,
        `Text exit code: ${textResult.exitCode}`,
        `DOT exit code: ${dotResult.exitCode}`,
    ];

    if (summary.moduleCount !== null) {
        lines.push(`Modules analyzed: ${summary.moduleCount}`);
    }

    if (summary.violationCount !== null) {
        lines.push(`Violations reported: ${summary.violationCount}`);
    }

    lines.push("");
    lines.push(`Text report: ${path.relative(process.cwd(), textFile)}`);
    lines.push(`Graph report: ${path.relative(process.cwd(), dotFile)}`);
    lines.push("");

    await writeJson(summaryFile, summary);
    await writeText(noteFile, `${lines.join("\n")}\n`);

    printSection("TypeScript Dependencies");
    console.log(lines.join("\n"));
    advisoryExitCode();
}

async function collectScanPaths() {
    const roots = ["apps/web/src"];
    const packageDirs = await fsp.readdir(path.join(process.cwd(), "packages"), { withFileTypes: true });

    for (const entry of packageDirs) {
        if (!entry.isDirectory()) {
            continue;
        }

        const sourceDir = path.join(process.cwd(), "packages", entry.name, "src");
        try {
            const stat = await fsp.stat(sourceDir);
            if (stat.isDirectory()) {
                roots.push(path.join("packages", entry.name, "src"));
            }
        } catch (_error) {
            // Ignore packages without a source root.
        }
    }

    return roots.sort();
}

main().catch((error) => {
    console.error(error instanceof Error ? error.stack || error.message : String(error));
    process.exitCode = 1;
});
