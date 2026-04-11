import path from "node:path";
import fsp from "node:fs/promises";
import {
    ANALYSIS_DIR,
    advisoryExitCode,
    collectFiles,
    ensureDir,
    findExecutable,
    formatInstallHint,
    printSection,
    runCommand,
    writeJson,
    writeText,
} from "./common.mjs";

const OUTPUT_DIR = path.join(ANALYSIS_DIR, "rust-metrics");

async function main() {
    await ensureDir(OUTPUT_DIR);

    const rustFiles = await collectFiles(["apps", "crates"], (_fullPath, relativePath) => relativePath.endsWith(".rs"));
    const fileMetrics = [];

    for (const file of rustFiles) {
        const source = await fsp.readFile(file.fullPath, "utf8");
        fileMetrics.push(analyzeRustFile(file.relativePath, source));
    }

    const totals = summarizeMetrics(fileMetrics);
    const hotspots = [...fileMetrics]
        .sort((left, right) => {
            if (right.complexityScore !== left.complexityScore) {
                return right.complexityScore - left.complexityScore;
            }
            return right.loc - left.loc;
        })
        .slice(0, 15);

    const binary = findExecutable(["rust-code-analysis-cli", "rust-code-analysis"]);
    let externalTool = {
        tool: binary ? path.basename(binary) : "rust-code-analysis",
        installed: Boolean(binary),
        installHint: formatInstallHint(
            "rust-code-analysis",
            "cargo install rust-code-analysis-cli",
        ),
        command: null,
        exitCode: null,
        helpFile: null,
    };

    if (binary) {
        const help = runCommand(binary, ["--help"], { allowFailure: true });
        const helpFile = path.join(OUTPUT_DIR, "rust-code-analysis.help.txt");
        await writeText(helpFile, `${help.stdout || ""}${help.stderr || ""}`);
        externalTool = {
            ...externalTool,
            command: help.command,
            exitCode: help.exitCode,
            helpFile,
        };
    }

    const summary = {
        generatedAt: new Date().toISOString(),
        advisory: true,
        source: {
            heuristicMetrics: true,
            externalToolPrepared: true,
        },
        totals,
        hotspots,
        files: fileMetrics,
        externalTool,
    };

    const summaryFile = path.join(OUTPUT_DIR, "summary.json");
    const textFile = path.join(OUTPUT_DIR, "summary.txt");

    const lines = [
        "Rust metrics analysis",
        "",
        `Rust files: ${totals.fileCount}`,
        `Approximate LOC: ${totals.loc}`,
        `Approximate functions: ${totals.functionCount}`,
        `Average LOC per file: ${totals.averageLocPerFile.toFixed(1)}`,
        "",
        "Hotspots:",
        ...hotspots.map((file) => `- ${file.path}: score=${file.complexityScore}, loc=${file.loc}, fn=${file.functionCount}`),
        "",
    ];

    if (!binary) {
        lines.push(externalTool.installHint);
        lines.push("");
    }

    await writeJson(summaryFile, summary);
    await writeText(textFile, `${lines.join("\n")}\n`);

    printSection("Rust Metrics");
    console.log(lines.join("\n"));
    advisoryExitCode();
}

function analyzeRustFile(relativePath, source) {
    const lines = source.split("\n");
    const nonEmptyLines = lines.filter((line) => line.trim() !== "");
    const codeLines = lines.filter((line) => {
        const trimmed = line.trim();
        return trimmed !== "" && !trimmed.startsWith("//");
    });
    const functionMatches = source.match(/^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+[A-Za-z0-9_]+/gm) || [];
    const controlMatches = source.match(/\b(if|else if|match|for|while|loop)\b|&&|\|\|/g) || [];
    const resultMatches = source.match(/\b(Result|Option)\b/g) || [];

    const complexityScore = controlMatches.length + functionMatches.length + Math.round(codeLines.length / 40);

    return {
        path: relativePath,
        loc: codeLines.length,
        rawLineCount: lines.length,
        nonEmptyLineCount: nonEmptyLines.length,
        functionCount: functionMatches.length,
        controlFlowCount: controlMatches.length,
        errorTypeCount: resultMatches.length,
        complexityScore,
    };
}

function summarizeMetrics(fileMetrics) {
    const loc = fileMetrics.reduce((sum, file) => sum + file.loc, 0);
    const functionCount = fileMetrics.reduce((sum, file) => sum + file.functionCount, 0);

    return {
        fileCount: fileMetrics.length,
        loc,
        functionCount,
        averageLocPerFile: fileMetrics.length === 0 ? 0 : loc / fileMetrics.length,
    };
}

main().catch((error) => {
    console.error(error instanceof Error ? error.stack || error.message : String(error));
    process.exitCode = 1;
});
