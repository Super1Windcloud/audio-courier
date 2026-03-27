import { spawn } from "node:child_process";
import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { config as loadDotenv } from "dotenv";

const rootDir = process.cwd();
const bundleDir = path.join(
	rootDir,
	"src-tauri",
	"target",
	"release",
	"bundle",
);
const tauriConfigPath = path.join(rootDir, "src-tauri", "tauri.conf.json");
const cargoTomlPath = path.join(rootDir, "src-tauri", "Cargo.toml");
const packageJsonPath = path.join(rootDir, "package.json");
const defaultPrivateKeyPath = path.join(
	rootDir,
	".tauri",
	"audio-courier_signer.key",
);
const rootEnvPath = path.join(rootDir, ".env");
const tauriEnvPath = path.join(rootDir, "src-tauri", ".env");
const updaterMetadataDir = path.join(rootDir, "updater");
const updaterMetadataPath = path.join(updaterMetadataDir, "latest.json");

const mode = process.argv[2] ?? "all";

void main().catch((error: unknown) => {
	console.error(error instanceof Error ? error.message : String(error));
	process.exitCode = 1;
});

async function main() {
	loadEnvFiles();
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

function loadEnvFiles() {
	loadDotenv({ path: rootEnvPath });
	loadDotenv({ path: tauriEnvPath, override: false });
}

function assertMode(value: string) {
	if (!["all", "build", "publish"].includes(value)) {
		throw new Error("usage: tsx scripts/release.ts [build|publish]");
	}
}

async function loadVersions() {
	const [packageJsonRaw, tauriConfigRaw, cargoToml] = await Promise.all([
		readFile(packageJsonPath, "utf8"),
		readFile(tauriConfigPath, "utf8"),
		readFile(cargoTomlPath, "utf8"),
	]);

	const packageJson = JSON.parse(packageJsonRaw) as {
		version?: string;
		repository?: string;
		homepage?: string;
	};
	const tauriConfig = JSON.parse(tauriConfigRaw) as {
		version?: string;
		productName?: string;
	};
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

function assertVersions(versions: {
	packageVersion?: string;
	tauriVersion?: string;
	cargoVersion?: string;
}) {
	const { packageVersion, tauriVersion, cargoVersion } = versions;
	if (!packageVersion || !tauriVersion || !cargoVersion) {
		throw new Error(
			"package.json, tauri.conf.json and Cargo.toml must all declare a version",
		);
	}

	if (packageVersion !== tauriVersion || packageVersion !== cargoVersion) {
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

async function buildRelease(version: string) {
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

async function publishRelease(version: string) {
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

	if (artifactContext.uploads.length === 0 || !artifactContext.manifestEntry) {
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
		path: updaterMetadataPath,
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

	await writeLatestJson(nextLatest);

	await deleteReleaseAssetByName({
		token,
		release,
		name: "latest.json",
	});

	return {
		tagName,
		uploadedAssets: artifactContext.uploads.map((item) => item.name),
	};
}

async function resolveRepository() {
	const packageJsonRaw = await readFile(packageJsonPath, "utf8");
	const cargoToml = await readFile(cargoTomlPath, "utf8");
	const packageJson = JSON.parse(packageJsonRaw) as {
		repository?: string;
		homepage?: string;
	};
	const repositoryFromCargo = cargoToml.match(
		/^repository\s*=\s*"([^"]+)"/m,
	)?.[1];

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

function parseGithubRepository(value: string | undefined | null) {
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

async function loadReleaseNotes(version: string) {
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
	const targetIndex = args.findIndex(
		(item) => item === "--target" || item === "-t",
	);
	if (targetIndex >= 0 && args[targetIndex + 1]) {
		return args[targetIndex + 1];
	}

	return process.env.RELEASE_TARGET_TRIPLE ?? null;
}

async function collectArtifacts(input: {
	bundleDir: string;
	repository: string;
	tagName: string;
	targetTriple: string | null;
}) {
	const files = await listFiles(input.bundleDir);
	const preferred = findPreferredArtifact(files, input.targetTriple);

	if (!preferred) {
		return {
			uploads: [] as UploadAsset[],
			manifestEntry: null,
		};
	}

	const uploads = preferred.relatedFiles.map(async (filePath) => ({
		name: path.basename(filePath),
		contentType: contentTypeFor(filePath),
		buffer: await readFile(filePath),
	}));

	return {
		uploads: await Promise.all(uploads),
		manifestEntry: {
			platform: preferred.platform,
			signature: (await readFile(preferred.signaturePath, "utf8")).trim(),
			url: `https://github.com/${input.repository}/releases/download/${input.tagName}/${encodeURIComponent(
				sanitizeGitHubAssetName(path.basename(preferred.artifactPath)),
			)}`,
		},
	};
}

function findPreferredArtifact(files: string[], targetTriple: string | null) {
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
		.filter((item): item is PreferredArtifact => item !== null);

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

function classifyArtifact(filePath: string, targetTriple: string | null) {
	const fileName = path.basename(filePath);

	if (fileName.endsWith(".AppImage")) {
		return {
			kind: "appimage",
			platform: platformKey("linux", targetTriple),
		} as const;
	}

	if (fileName.endsWith(".app.tar.gz")) {
		return {
			kind: "app-tar",
			platform: platformKey("darwin", targetTriple),
		} as const;
	}

	if (fileName.endsWith(".msi")) {
		return {
			kind: "msi",
			platform: platformKey("windows", targetTriple),
		} as const;
	}

	if (fileName.endsWith(".exe")) {
		return {
			kind: "nsis",
			platform: platformKey("windows", targetTriple),
		} as const;
	}

	return null;
}

function platformKey(osName: string, targetTriple: string | null) {
	if (targetTriple) {
		return `${osName}-${archFromTargetTriple(targetTriple)}`;
	}

	return `${osName}-${archFromNode(process.arch)}`;
}

function archFromTargetTriple(targetTriple: string) {
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

	throw new Error(
		`unsupported target triple for updater manifest: ${targetTriple}`,
	);
}

function archFromNode(arch: string) {
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
			throw new Error(
				`unsupported node architecture for updater manifest: ${arch}`,
			);
	}
}

async function listFiles(directory: string): Promise<string[]> {
	const entries = await readdir(directory, { withFileTypes: true });
	const files: string[] = [];

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

function contentTypeFor(filePath: string) {
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

async function ensureRelease(input: {
	token: string;
	repository: string;
	tagName: string;
	releaseName: string;
	releaseBody: string;
}) {
	const existing = await githubRequest<GitHubRelease>({
		token: input.token,
		url: `https://api.github.com/repos/${input.repository}/releases/tags/${encodeURIComponent(
			input.tagName,
		)}`,
		allow404: true,
	});

	if (existing.status === 200 && existing.json) {
		return existing.json;
	}

	const created = await githubRequest<GitHubRelease>({
		token: input.token,
		method: "POST",
		url: `https://api.github.com/repos/${input.repository}/releases`,
		body: {
			tag_name: input.tagName,
			name: input.releaseName,
			body: input.releaseBody,
			draft: false,
			prerelease: false,
			generate_release_notes: false,
			target_commitish: process.env.RELEASE_TARGET_COMMITISH ?? "main",
		},
	});

	if (!created.json) {
		throw new Error("failed to create GitHub release");
	}

	return created.json;
}

async function loadExistingLatestJson(input: { path: string }) {
	try {
		const content = await readFile(input.path, "utf8");
		return JSON.parse(content) as LatestJson;
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		if (
			message.includes("ENOENT") ||
			message.includes("no such file") ||
			message.includes("系统找不到指定的文件")
		) {
			return null;
		}

		throw error;
	}
}

async function writeLatestJson(content: LatestJson) {
	await mkdir(updaterMetadataDir, { recursive: true });
	await writeFile(
		updaterMetadataPath,
		`${JSON.stringify(content, null, 2)}\n`,
		"utf8",
	);
}

async function listReleaseAssets(input: {
	token: string;
	release: GitHubRelease;
}) {
	const response = await githubRequest<GitHubAsset[]>({
		token: input.token,
		url: `${input.release.url}/assets?per_page=100`,
	});

	return response.json ?? [];
}

async function deleteReleaseAssetByName(input: {
	token: string;
	release: GitHubRelease;
	name: string;
}) {
	input.release.assets = await listReleaseAssets({
		token: input.token,
		release: input.release,
	});
	const candidateNames = new Set([
		input.name,
		sanitizeGitHubAssetName(input.name),
	]);
	const matchingAssets =
		input.release.assets?.filter((item) => candidateNames.has(item.name)) ?? [];
	if (matchingAssets.length === 0) {
		return null;
	}

	for (const asset of matchingAssets) {
		await githubRequest({
			token: input.token,
			method: "DELETE",
			url: releaseAssetDeleteUrl(input.release, asset.id),
		});

		input.release.assets = input.release.assets.filter(
			(item) => item.id !== asset.id,
		);
	}

	return null;
}

async function uploadReleaseAsset(input: {
	token: string;
	release: GitHubRelease;
	upload: UploadAsset;
}) {
	const uploadUrl = input.release.upload_url.replace("{?name,label}", "");
	let response = await fetch(
		`${uploadUrl}?name=${encodeURIComponent(input.upload.name)}`,
		{
			method: "POST",
			headers: {
				Authorization: `Bearer ${input.token}`,
				"Content-Type": input.upload.contentType,
				Accept: "application/vnd.github+json",
				"User-Agent": "audio-courier-release-script",
			},
			body: input.upload.buffer,
		},
	);

	if (response.status === 422) {
		const errorText = await response.text();
		if (errorText.includes('"already_exists"')) {
			await deleteReleaseAssetByName({
				token: input.token,
				release: input.release,
				name: input.upload.name,
			});

			response = await fetch(
				`${uploadUrl}?name=${encodeURIComponent(input.upload.name)}`,
				{
					method: "POST",
					headers: {
						Authorization: `Bearer ${input.token}`,
						"Content-Type": input.upload.contentType,
						Accept: "application/vnd.github+json",
						"User-Agent": "audio-courier-release-script",
					},
					body: input.upload.buffer,
				},
			);
		} else {
			throw new Error(
				`failed to upload ${input.upload.name}: ${response.status} ${errorText}`,
			);
		}
	}

	if (!response.ok) {
		const errorText = await response.text();
		throw new Error(
			`failed to upload ${input.upload.name}: ${response.status} ${errorText}`,
		);
	}

	const asset = (await response.json()) as GitHubAsset;
	input.release.assets = [...(input.release.assets ?? []), asset];
}

async function githubRequest<T>(input: {
	token: string;
	url: string;
	method?: string;
	body?: unknown;
	allow404?: boolean;
}) {
	const response = await fetch(input.url, {
		method: input.method ?? "GET",
		headers: {
			Authorization: `Bearer ${input.token}`,
			Accept: "application/vnd.github+json",
			"Content-Type": "application/json",
			"User-Agent": "audio-courier-release-script",
		},
		body: input.body ? JSON.stringify(input.body) : undefined,
	});

	if (input.allow404 && response.status === 404) {
		return { status: 404, json: null as T | null };
	}

	if (!response.ok) {
		const errorText = await response.text();
		throw new Error(`github api failed: ${response.status} ${errorText}`);
	}

	const json =
		response.status === 204 ? null : ((await response.json()) as T | null);
	return { status: response.status, json };
}

function requireEnv(name: string) {
	const value = process.env[name];
	if (!value) {
		throw new Error(`missing required environment variable: ${name}`);
	}

	return value;
}

function splitArgs(value: string | undefined) {
	if (!value) {
		return [];
	}

	return (
		value
			.match(/(?:[^\s"]+|"[^"]*")+/g)
			?.map((item) => item.replace(/^"|"$/g, "")) ?? []
	);
}

function sanitizeGitHubAssetName(name: string) {
	return name.replace(/[^A-Za-z0-9._-]+/g, ".");
}

function releaseAssetDeleteUrl(release: GitHubRelease, assetId: number) {
	return release.url.replace(/\/releases\/\d+$/, `/releases/assets/${assetId}`);
}

async function runCommand(
	command: string,
	args: string[],
	options: {
		env?: NodeJS.ProcessEnv;
	} = {},
) {
	await new Promise<void>((resolve, reject) => {
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

			reject(
				new Error(`${command} ${args.join(" ")} failed with exit code ${code}`),
			);
		});

		child.on("error", reject);
	});
}

type UploadAsset = {
	name: string;
	contentType: string;
	buffer: Buffer;
};

type PreferredArtifact = {
	artifactPath: string;
	signaturePath: string;
	hasSignature: boolean;
	kind: "nsis" | "appimage" | "app-tar" | "msi";
	platform: string;
	relatedFiles: string[];
};

type GitHubAsset = {
	id: number;
	name: string;
	url: string;
};

type GitHubRelease = {
	id: number;
	url: string;
	assets_url: string;
	upload_url: string;
	assets: GitHubAsset[];
};

type LatestJson = {
	version: string;
	notes: string;
	pub_date: string;
	platforms: Record<
		string,
		{
			signature: string;
			url: string;
		}
	>;
};
