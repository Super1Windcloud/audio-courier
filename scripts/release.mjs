import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { spawn } from "node:child_process";

const rootDir = process.cwd();
const bundleDir = path.join(rootDir, "src-tauri", "target", "release", "bundle");
const tauriConfigPath = path.join(rootDir, "src-tauri", "tauri.conf.json");
const cargoTomlPath = path.join(rootDir, "src-tauri", "Cargo.toml");
const packageJsonPath = path.join(rootDir, "package.json");
const defaultPrivateKeyPath = path.join(
	rootDir,
	".tauri",
	"audio-courier_signer.key",
);

const mode = process.argv[2] ?? "all";

main().catch((error) => {
	console.error(error instanceof Error ? error.message : String(error));
	process.exitCode = 1;
});

async function main() {
	assertMode(mode);

	const versions = await loadVersions();
	assertVersions(versions);

	if (mode === "build" || mode === "all") {
		await buildRelease(versions.packageVersion);
	}

	if (mode === "publish" || mode === "all") {
		const releaseContext = await publishRelease(versions.packageVersion);
		console.log(
			`published ${releaseContext.tagName} with ${releaseContext.uploadedAssets.length} asset(s)`,
		);
	}
}

function assertMode(value) {
	if (!["all", "build", "publish"].includes(value)) {
		throw new Error("usage: node scripts/release.mjs [build|publish]");
	}
}

async function loadVersions() {
	const [packageJsonRaw, tauriConfigRaw, cargoToml] = await Promise.all([
		readFile(packageJsonPath, "utf8"),
		readFile(tauriConfigPath, "utf8"),
		readFile(cargoTomlPath, "utf8"),
	]);

	const packageJson = JSON.parse(packageJsonRaw);
	const tauriConfig = JSON.parse(tauriConfigRaw);
	const cargoVersionMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);

	if (!cargoVersionMatch) {
		throw new Error("unable to read version from src-tauri/Cargo.toml");
	}

	return {
		packageVersion: packageJson.version,
		tauriVersion: tauriConfig.version,
		cargoVersion: cargoVersionMatch[1],
		repository:
			process.env.GITHUB_REPOSITORY ??
			parseGithubRepository(
				packageJson.repository ??
					packageJson.homepage ??
					tauriConfig.productName ??
					"",
			),
	};
}

function assertVersions(versions) {
	const { packageVersion, tauriVersion, cargoVersion } = versions;
	if (!packageVersion || !tauriVersion || !cargoVersion) {
		throw new Error("package.json, tauri.conf.json and Cargo.toml must all declare a version");
	}

	if (
		packageVersion !== tauriVersion ||
		packageVersion !== cargoVersion
	) {
		throw new Error(
			[
				"version mismatch detected:",
				`package.json=${packageVersion}`,
				`src-tauri/tauri.conf.json=${tauriVersion}`,
				`src-tauri/Cargo.toml=${cargoVersion}`,
			].join(" "),
		);
	}
}

async function buildRelease(version) {
	ensureSigningEnv();
	const extraArgs = splitArgs(process.env.RELEASE_TAURI_ARGS);
	console.log(`building audio-courier ${version}`);
	await runCommand("pnpm", ["tauri", "build", "--ci", ...extraArgs], {
		env: {
			...process.env,
			CI: "true",
		},
	});
}

function ensureSigningEnv() {
	if (process.env.TAURI_SIGNING_PRIVATE_KEY) {
		return;
	}

	if (process.env.TAURI_SIGNING_PRIVATE_KEY_PATH) {
		process.env.TAURI_SIGNING_PRIVATE_KEY =
			process.env.TAURI_SIGNING_PRIVATE_KEY_PATH;
		return;
	}

	process.env.TAURI_SIGNING_PRIVATE_KEY = defaultPrivateKeyPath;
}

async function publishRelease(version) {
	const token = requireEnv("GITHUB_TOKEN");
	const repository = await resolveRepository();
	const tagName = process.env.RELEASE_TAG ?? `audio-courier-v${version}`;
	const releaseName = process.env.RELEASE_NAME ?? `audio-courier v${version}`;
	const releaseBody = await loadReleaseNotes(version);
	const targetTriple = resolveTargetTriple();

	const artifactContext = await collectArtifacts({
		bundleDir,
		repository,
		tagName,
		targetTriple,
	});

	if (artifactContext.uploads.length === 0) {
		throw new Error(
			"no updater artifacts found. build first with a signing key so .sig files are generated",
		);
	}

	const release = await ensureRelease({
		token,
		repository,
		tagName,
		releaseName,
		releaseBody,
	});

	const currentLatest = await loadExistingLatestJson({
		token,
		release,
	});

	for (const upload of artifactContext.uploads) {
		await deleteReleaseAssetByName({
			token,
			release,
			name: upload.name,
		});

		await uploadReleaseAsset({
			token,
			release,
			upload,
		});
	}

	const nextLatest = {
		version,
		notes: releaseBody,
		pub_date: new Date().toISOString(),
		platforms: {
			...(currentLatest?.platforms ?? {}),
			[artifactContext.manifestEntry.platform]: {
				signature: artifactContext.manifestEntry.signature,
				url: artifactContext.manifestEntry.url,
			},
		},
	};

	const latestBuffer = Buffer.from(`${JSON.stringify(nextLatest, null, 2)}\n`);

	await deleteReleaseAssetByName({
		token,
		release,
		name: "latest.json",
	});

	await uploadReleaseAsset({
		token,
		release,
		upload: {
			name: "latest.json",
			contentType: "application/json; charset=utf-8",
			buffer: latestBuffer,
		},
	});

	return {
		tagName,
		uploadedAssets: [
			...artifactContext.uploads.map((item) => item.name),
			"latest.json",
		],
	};
}

async function resolveRepository() {
	const packageJsonRaw = await readFile(packageJsonPath, "utf8");
	const cargoToml = await readFile(cargoTomlPath, "utf8");
	const packageJson = JSON.parse(packageJsonRaw);
	const repositoryFromCargo = cargoToml.match(/^repository\s*=\s*"([^"]+)"/m)?.[1];

	const repository = parseGithubRepository(
		process.env.GITHUB_REPOSITORY ??
			repositoryFromCargo ??
			packageJson.repository ??
			packageJson.homepage ??
			"",
	);

	if (!repository) {
		throw new Error(
			"set GITHUB_REPOSITORY=owner/repo or add a valid GitHub repository URL to package.json/Cargo.toml",
		);
	}

	return repository;
}

function parseGithubRepository(value) {
	if (!value || typeof value !== "string") {
		return null;
	}

	const normalized = value
		.replace(/^git\+/, "")
		.replace(/\.git$/, "")
		.replace(/^https?:\/\/github\.com\//i, "")
		.replace(/^github\.com\//i, "")
		.replace(/^github\//i, "");

	const match = normalized.match(/^([^/\s]+)\/([^/\s]+)$/);
	if (!match) {
		return null;
	}

	return `${match[1]}/${match[2]}`;
}

async function loadReleaseNotes(version) {
	const notesFile = process.env.RELEASE_NOTES_FILE;
	if (notesFile) {
		return (await readFile(path.resolve(rootDir, notesFile), "utf8")).trim();
	}

	if (process.env.RELEASE_NOTES) {
		return process.env.RELEASE_NOTES.trim();
	}

	return `Release ${version}`;
}

function resolveTargetTriple() {
	const args = splitArgs(process.env.RELEASE_TAURI_ARGS);
	const targetIndex = args.findIndex((item) => item === "--target" || item === "-t");
	if (targetIndex >= 0 && args[targetIndex + 1]) {
		return args[targetIndex + 1];
	}

	return process.env.RELEASE_TARGET_TRIPLE ?? null;
}

async function collectArtifacts({ bundleDir, repository, tagName, targetTriple }) {
	const files = await listFiles(bundleDir);
	const preferred = findPreferredArtifact(files, targetTriple);

	if (!preferred) {
		return {
			uploads: [],
			manifestEntry: null,
		};
	}

	const uploads = preferred.relatedFiles.map((filePath) => ({
		name: path.basename(filePath),
		contentType: contentTypeFor(filePath),
		bufferPromise: readFile(filePath),
	}));

	const resolvedUploads = [];
	for (const upload of uploads) {
		resolvedUploads.push({
			...upload,
			buffer: await upload.bufferPromise,
		});
	}

	return {
		uploads: resolvedUploads,
		manifestEntry: {
			platform: preferred.platform,
			signature: (await readFile(preferred.signaturePath, "utf8")).trim(),
			url: `https://github.com/${repository}/releases/download/${tagName}/${encodeURIComponent(
				path.basename(preferred.artifactPath),
			)}`,
		},
	};
}

function findPreferredArtifact(files, targetTriple) {
	const artifacts = files
		.filter((filePath) => !filePath.endsWith(".sig"))
		.map((filePath) => {
			const signaturePath = `${filePath}.sig`;
			return {
				artifactPath: filePath,
				signaturePath,
				hasSignature: files.includes(signaturePath),
			};
		})
		.filter((item) => item.hasSignature)
		.map((item) => {
			const metadata = classifyArtifact(item.artifactPath, targetTriple);
			return metadata ? { ...item, ...metadata } : null;
		})
		.filter(Boolean);

	const preferredOrder = ["nsis", "appimage", "app-tar", "msi"];
	for (const kind of preferredOrder) {
		const match = artifacts.find((item) => item.kind === kind);
		if (match) {
			return {
				...match,
				relatedFiles: [match.artifactPath, match.signaturePath],
			};
		}
	}

	return null;
}

function classifyArtifact(filePath, targetTriple) {
	const fileName = path.basename(filePath);

	if (fileName.endsWith(".AppImage")) {
		return {
			kind: "appimage",
			platform: platformKey("linux", targetTriple),
		};
	}

	if (fileName.endsWith(".app.tar.gz")) {
		return {
			kind: "app-tar",
			platform: platformKey("darwin", targetTriple),
		};
	}

	if (fileName.endsWith(".msi")) {
		return {
			kind: "msi",
			platform: platformKey("windows", targetTriple),
		};
	}

	if (fileName.endsWith(".exe")) {
		return {
			kind: "nsis",
			platform: platformKey("windows", targetTriple),
		};
	}

	return null;
}

function platformKey(osName, targetTriple) {
	if (targetTriple) {
		return `${osName}-${archFromTargetTriple(targetTriple)}`;
	}

	return `${osName}-${archFromNode(process.arch)}`;
}

function archFromTargetTriple(targetTriple) {
	if (targetTriple.startsWith("x86_64-")) {
		return "x86_64";
	}

	if (targetTriple.startsWith("aarch64-")) {
		return "aarch64";
	}

	if (targetTriple.startsWith("i686-")) {
		return "i686";
	}

	if (targetTriple.startsWith("armv7-")) {
		return "armv7";
	}

	throw new Error(`unsupported target triple for updater manifest: ${targetTriple}`);
}

function archFromNode(arch) {
	switch (arch) {
		case "x64":
			return "x86_64";
		case "arm64":
			return "aarch64";
		case "ia32":
			return "i686";
		case "arm":
			return "armv7";
		default:
			throw new Error(`unsupported node architecture for updater manifest: ${arch}`);
	}
}

async function listFiles(directory) {
	const entries = await readdir(directory, { withFileTypes: true });
	const files = [];

	for (const entry of entries) {
		const fullPath = path.join(directory, entry.name);
		if (entry.isDirectory()) {
			files.push(...(await listFiles(fullPath)));
			continue;
		}

		files.push(fullPath);
	}

	return files;
}

function contentTypeFor(filePath) {
	if (filePath.endsWith(".json")) {
		return "application/json; charset=utf-8";
	}

	if (filePath.endsWith(".sig")) {
		return "text/plain; charset=utf-8";
	}

	if (filePath.endsWith(".tar.gz")) {
		return "application/gzip";
	}

	return "application/octet-stream";
}

async function ensureRelease({
	token,
	repository,
	tagName,
	releaseName,
	releaseBody,
}) {
	const existing = await githubRequest({
		token,
		url: `https://api.github.com/repos/${repository}/releases/tags/${encodeURIComponent(
			tagName,
		)}`,
		allow404: true,
	});

	if (existing.status === 200) {
		return existing.json;
	}

	const created = await githubRequest({
		token,
		method: "POST",
		url: `https://api.github.com/repos/${repository}/releases`,
		body: {
			tag_name: tagName,
			name: releaseName,
			body: releaseBody,
			draft: false,
			prerelease: false,
			generate_release_notes: false,
			target_commitish: process.env.RELEASE_TARGET_COMMITISH ?? "main",
		},
	});

	return created.json;
}

async function loadExistingLatestJson({ token, release }) {
	const latestAsset = release.assets?.find((asset) => asset.name === "latest.json");
	if (!latestAsset) {
		return null;
	}

	const response = await fetch(latestAsset.url, {
		headers: {
			Authorization: `Bearer ${token}`,
			Accept: "application/octet-stream",
			"User-Agent": "audio-courier-release-script",
		},
	});

	if (!response.ok) {
		throw new Error(`failed to download existing latest.json: ${response.status}`);
	}

	return response.json();
}

async function deleteReleaseAssetByName({ token, release, name }) {
	const asset = release.assets?.find((item) => item.name === name);
	if (!asset) {
		return;
	}

	await githubRequest({
		token,
		method: "DELETE",
		url: `https://api.github.com/repos/${release.repository_url.split("/repos/")[1]}/releases/assets/${asset.id}`,
	});

	release.assets = release.assets.filter((item) => item.id !== asset.id);
}

async function uploadReleaseAsset({ token, release, upload }) {
	const uploadUrl = release.upload_url.replace("{?name,label}", "");
	const response = await fetch(
		`${uploadUrl}?name=${encodeURIComponent(upload.name)}`,
		{
			method: "POST",
			headers: {
				Authorization: `Bearer ${token}`,
				"Content-Type": upload.contentType,
				Accept: "application/vnd.github+json",
				"User-Agent": "audio-courier-release-script",
			},
			body: upload.buffer,
		},
	);

	if (!response.ok) {
		const errorText = await response.text();
		throw new Error(`failed to upload ${upload.name}: ${response.status} ${errorText}`);
	}

	const asset = await response.json();
	release.assets = [...(release.assets ?? []), asset];
}

async function githubRequest({
	token,
	url,
	method = "GET",
	body,
	allow404 = false,
}) {
	const response = await fetch(url, {
		method,
		headers: {
			Authorization: `Bearer ${token}`,
			Accept: "application/vnd.github+json",
			"Content-Type": "application/json",
			"User-Agent": "audio-courier-release-script",
		},
		body: body ? JSON.stringify(body) : undefined,
	});

	if (allow404 && response.status === 404) {
		return { status: 404, json: null };
	}

	if (!response.ok) {
		const errorText = await response.text();
		throw new Error(`github api failed: ${response.status} ${errorText}`);
	}

	const json = response.status === 204 ? null : await response.json();
	return { status: response.status, json };
}

function requireEnv(name) {
	const value = process.env[name];
	if (!value) {
		throw new Error(`missing required environment variable: ${name}`);
	}

	return value;
}

function splitArgs(value) {
	if (!value) {
		return [];
	}

	return value.match(/(?:[^\s"]+|"[^"]*")+/g)?.map((item) => item.replace(/^"|"$/g, "")) ?? [];
}

async function runCommand(command, args, options = {}) {
	await new Promise((resolve, reject) => {
		const child = spawn(command, args, {
			stdio: "inherit",
			shell: process.platform === "win32",
			...options,
		});

		child.on("exit", (code) => {
			if (code === 0) {
				resolve();
				return;
			}

			reject(new Error(`${command} ${args.join(" ")} failed with exit code ${code}`));
		});

		child.on("error", reject);
	});
}
