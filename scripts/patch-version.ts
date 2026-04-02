import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const rootDir = process.cwd();
const packageJsonPath = path.join(rootDir, "package.json");
const tauriConfigPath = path.join(rootDir, "src-tauri", "tauri.conf.json");
const cargoTomlPath = path.join(rootDir, "src-tauri", "Cargo.toml");

void main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});

async function main() {
  const [packageJsonRaw, tauriConfigRaw, cargoTomlRaw] = await Promise.all([
    readFile(packageJsonPath, "utf8"),
    readFile(tauriConfigPath, "utf8"),
    readFile(cargoTomlPath, "utf8"),
  ]);

  const packageJson = JSON.parse(packageJsonRaw) as { version?: string };
  const tauriConfig = JSON.parse(tauriConfigRaw) as { version?: string };
  const cargoVersionMatch = cargoTomlRaw.match(/^version\s*=\s*"([^"]+)"/m);

  if (!packageJson.version || !tauriConfig.version || !cargoVersionMatch?.[1]) {
    throw new Error(
      "package.json, src-tauri/tauri.conf.json and src-tauri/Cargo.toml must all declare a version",
    );
  }

  if (
    packageJson.version !== tauriConfig.version ||
    packageJson.version !== cargoVersionMatch[1]
  ) {
    throw new Error(
      [
        "version mismatch detected:",
        `package.json=${packageJson.version}`,
        `src-tauri/tauri.conf.json=${tauriConfig.version}`,
        `src-tauri/Cargo.toml=${cargoVersionMatch[1]}`,
      ].join(" "),
    );
  }

  const currentVersion = packageJson.version;
  const nextVersion = decrementPatchVersion(currentVersion);

  packageJson.version = nextVersion;
  tauriConfig.version = nextVersion;

  const nextCargoToml = cargoTomlRaw.replace(
    /^version\s*=\s*"([^"]+)"/m,
    `version = "${nextVersion}"`,
  );

  await Promise.all([
    writeFile(
      packageJsonPath,
      `${JSON.stringify(packageJson, null, 2)}\n`,
      "utf8",
    ),
    writeFile(
      tauriConfigPath,
      `${JSON.stringify(tauriConfig, null, 2)}\n`,
      "utf8",
    ),
    writeFile(cargoTomlPath, nextCargoToml, "utf8"),
  ]);

  console.log(`version rollback: ${currentVersion} -> ${nextVersion}`);
}

function decrementPatchVersion(version: string) {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)$/);
  if (!match) {
    throw new Error(`unsupported version format: ${version}`);
  }

  const [, major, minor, patch] = match;
  const newPatch = Number.parseInt(patch, 10) - 1;

  if (newPatch < 0) {
    throw new Error(`cannot decrement version below 0: ${version}`);
  }

  return `${major}.${minor}.${newPatch}`;
}
