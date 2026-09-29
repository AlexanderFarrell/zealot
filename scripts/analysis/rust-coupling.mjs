import path from "node:path";
import {
    ANALYSIS_DIR,
    advisoryExitCode,
    ensureDir,
    formatInstallHint,
    printSection,
    runCommand,
    writeJson,
    writeText,
    findExecutable,
} from "./common.mjs";

const OUTPUT_DIR = path.join(ANALYSIS_DIR, "rust-coupling");

const ALLOWED_INTERNAL_DEPS = {
    "zealot-domain": [],
    "zealot-app": ["zealot-domain"],
    "zealot-api": ["zealot-domain", "zealot-app"],
    "zealot-infra": ["zealot-domain", "zealot-app"],
    "zealot-server": ["zealot-domain", "zealot-app", "zealot-api", "zealot-infra"],
    "zealot-cli": ["zealot-domain"],
    "zealot-tui": ["zealot-domain"],
    "zealot-mcp": ["zealot-domain"],
};

async function main() {
    await ensureDir(OUTPUT_DIR);

    const metadataResult = runCommand("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
        allowFailure: false,
    });

    const metadata = JSON.parse(metadataResult.stdout);
    const workspaceMembers = new Set(metadata.workspace_members);
    const internalPackages = metadata.packages
        .filter((pkg) => workspaceMembers.has(pkg.id))
        .map((pkg) => ({
            name: pkg.name,
            manifestPath: pkg.manifest_path,
            internalDeps: pkg.dependencies
                .filter((dep) => dep.path && Object.prototype.hasOwnProperty.call(ALLOWED_INTERNAL_DEPS, dep.name))
                .map((dep) => dep.name)
                .sort(),
        }))
        .sort((left, right) => left.name.localeCompare(right.name));

    const internalNames = new Set(internalPackages.map((pkg) => pkg.name));
    const violations = [];

    for (const pkg of internalPackages) {
        const allowed = new Set(ALLOWED_INTERNAL_DEPS[pkg.name] || []);

        for (const dependencyName of pkg.internalDeps) {
            if (!allowed.has(dependencyName)) {
                violations.push({
                    package: pkg.name,
                    dependency: dependencyName,
                    rule: "forbidden-internal-dependency",
                });
            }
        }

        for (const allowedName of allowed) {
            if (!internalNames.has(allowedName)) {
                violations.push({
                    package: pkg.name,
                    dependency: allowedName,
                    rule: "unknown-allowed-dependency",
                });
            }
        }
    }

    const cycles = findCycles(
        Object.fromEntries(internalPackages.map((pkg) => [pkg.name, pkg.internalDeps])),
    );

    const cargoCouplingBinary = findExecutable(["cargo-coupling"]);
    let externalTool = {
        tool: "cargo-coupling",
        installed: Boolean(cargoCouplingBinary),
        installHint: formatInstallHint("cargo-coupling", "cargo install cargo-coupling"),
        command: null,
        exitCode: null,
        stdoutFile: null,
        stderrFile: null,
    };

    if (cargoCouplingBinary) {
        const result = runCommand(cargoCouplingBinary, [], { allowFailure: true });
        const stdoutFile = path.join(OUTPUT_DIR, "cargo-coupling.stdout.txt");
        const stderrFile = path.join(OUTPUT_DIR, "cargo-coupling.stderr.txt");
        await writeText(stdoutFile, result.stdout || "");
        await writeText(stderrFile, result.stderr || "");
        externalTool = {
            ...externalTool,
            command: result.command,
            exitCode: result.exitCode,
            stdoutFile,
            stderrFile,
        };
    }

    const summary = {
        generatedAt: new Date().toISOString(),
        advisory: true,
        workspacePackages: internalPackages,
        boundaryModel: ALLOWED_INTERNAL_DEPS,
        violationCount: violations.length,
        violations,
        cycleCount: cycles.length,
        cycles,
        externalTool,
    };

    const lines = [
        "Rust internal dependency analysis",
        "",
        `Workspace packages: ${internalPackages.length}`,
        `Boundary violations: ${violations.length}`,
        `Cycles: ${cycles.length}`,
        "",
        "Allowed internal dependency model:",
        ...Object.entries(ALLOWED_INTERNAL_DEPS).map(([pkg, allowed]) => {
            const text = allowed.length === 0 ? "(none)" : allowed.join(", ");
            return `- ${pkg} -> ${text}`;
        }),
        "",
    ];

    if (violations.length > 0) {
        lines.push("Violations:");
        for (const violation of violations) {
            lines.push(`- ${violation.package} -> ${violation.dependency} (${violation.rule})`);
        }
        lines.push("");
    }

    if (cycles.length > 0) {
        lines.push("Cycles:");
        for (const cycle of cycles) {
            lines.push(`- ${cycle.join(" -> ")}`);
        }
        lines.push("");
    }

    if (!cargoCouplingBinary) {
        lines.push(externalTool.installHint);
        lines.push("");
    }

    const summaryFile = path.join(OUTPUT_DIR, "summary.json");
    const textFile = path.join(OUTPUT_DIR, "summary.txt");
    await writeJson(summaryFile, summary);
    await writeText(textFile, `${lines.join("\n")}\n`);

    printSection("Rust Coupling");
    console.log(lines.join("\n"));
    advisoryExitCode();
}

function findCycles(graph) {
    const cycles = [];
    const seen = new Set();

    function visit(node, stack, active) {
        active.add(node);
        stack.push(node);

        for (const next of graph[node] || []) {
            if (!graph[next]) {
                continue;
            }

            const cycleIndex = stack.indexOf(next);
            if (cycleIndex >= 0) {
                const cycle = [...stack.slice(cycleIndex), next];
                const signature = normalizeCycle(cycle).join("|");
                if (!seen.has(signature)) {
                    seen.add(signature);
                    cycles.push(normalizeCycle(cycle));
                }
                continue;
            }

            if (!active.has(next)) {
                visit(next, stack, active);
            }
        }

        stack.pop();
        active.delete(node);
    }

    for (const node of Object.keys(graph)) {
        visit(node, [], new Set());
    }

    return cycles.sort((left, right) => left.join("|").localeCompare(right.join("|")));
}

function normalizeCycle(cycle) {
    const closed = cycle[cycle.length - 1] === cycle[0] ? cycle.slice(0, -1) : [...cycle];
    const rotations = closed.map((_, index) => [...closed.slice(index), ...closed.slice(0, index)]);
    const normalized = rotations.sort((left, right) => left.join("|").localeCompare(right.join("|")))[0];
    return [...normalized, normalized[0]];
}

main().catch((error) => {
    console.error(error instanceof Error ? error.stack || error.message : String(error));
    process.exitCode = 1;
});
