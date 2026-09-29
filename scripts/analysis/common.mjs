import fs from "node:fs";
import fsp from "node:fs/promises";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));

export const ROOT_DIR = path.resolve(SCRIPT_DIR, "..", "..");
export const ANALYSIS_DIR = path.join(ROOT_DIR, "artifacts", "analysis");

export function repoPath(...segments) {
    return path.join(ROOT_DIR, ...segments);
}

export async function ensureDir(dirPath) {
    await fsp.mkdir(dirPath, { recursive: true });
}

export async function resetDir(dirPath) {
    await fsp.rm(dirPath, { recursive: true, force: true });
    await ensureDir(dirPath);
}

export async function writeText(filePath, content) {
    await ensureDir(path.dirname(filePath));
    await fsp.writeFile(filePath, content, "utf8");
}

export async function writeJson(filePath, value) {
    await writeText(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

export async function readJsonIfExists(filePath) {
    try {
        const raw = await fsp.readFile(filePath, "utf8");
        return JSON.parse(raw);
    } catch (error) {
        if (error && typeof error === "object" && "code" in error && error.code === "ENOENT") {
            return null;
        }
        throw error;
    }
}

export function fileExists(filePath) {
    return fs.existsSync(filePath);
}

export function findExecutable(candidates) {
    for (const candidate of candidates) {
        if (candidate.includes(path.sep) && fileExists(candidate)) {
            return candidate;
        }

        const resolved = spawnSync("bash", ["-lc", `command -v ${shellEscape(candidate)}`], {
            cwd: ROOT_DIR,
            encoding: "utf8",
        });

        if (resolved.status === 0) {
            const value = resolved.stdout.trim();
            if (value) {
                return value;
            }
        }
    }

    return null;
}

export function runCommand(command, args, options = {}) {
    const {
        cwd = ROOT_DIR,
        env = process.env,
        allowFailure = true,
        trim = false,
    } = options;

    const result = spawnSync(command, args, {
        cwd,
        env,
        encoding: "utf8",
    });

    const response = {
        ok: result.status === 0,
        exitCode: result.status ?? 1,
        stdout: trim ? (result.stdout || "").trim() : (result.stdout || ""),
        stderr: trim ? (result.stderr || "").trim() : (result.stderr || ""),
        error: result.error ? String(result.error.message || result.error) : null,
        command: [command, ...args].join(" "),
    };

    if (!allowFailure && !response.ok) {
        const error = new Error(`Command failed: ${response.command}`);
        error.result = response;
        throw error;
    }

    return response;
}

export function advisoryExitCode() {
    process.exitCode = 0;
}

export function printSection(title) {
    console.log(`\n== ${title} ==`);
}

export function shellEscape(value) {
    return `'${String(value).replace(/'/g, `'\\''`)}'`;
}

export function formatInstallHint(toolName, command) {
    return `${toolName} is not installed. Install it with: ${command}`;
}

export async function collectFiles(rootDirs, predicate) {
    const results = [];

    async function walk(currentPath) {
        let entries;

        try {
            entries = await fsp.readdir(currentPath, { withFileTypes: true });
        } catch (error) {
            if (error && typeof error === "object" && "code" in error && error.code === "ENOENT") {
                return;
            }
            throw error;
        }

        for (const entry of entries) {
            const fullPath = path.join(currentPath, entry.name);
            const relativePath = path.relative(ROOT_DIR, fullPath);

            if (entry.isDirectory()) {
                if (shouldSkipDirectory(relativePath)) {
                    continue;
                }
                await walk(fullPath);
                continue;
            }

            if (predicate(fullPath, relativePath)) {
                results.push({ fullPath, relativePath });
            }
        }
    }

    for (const dir of rootDirs) {
        await walk(repoPath(dir));
    }

    return results.sort((left, right) => left.relativePath.localeCompare(right.relativePath));
}

function shouldSkipDirectory(relativePath) {
    return [
        "node_modules",
        "target",
        "dist",
        "artifacts",
        ".git",
        "test/playwright/node_modules",
        "test/playwright/playwright-report",
    ].some((part) => relativePath === part || relativePath.startsWith(`${part}/`) || relativePath.includes(`/${part}/`));
}
