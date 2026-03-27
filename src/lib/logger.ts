import { debug, error, info, warn } from "@tauri-apps/plugin-log";

let didInit = false;

export function serializeError(errorLike: unknown) {
	if (errorLike instanceof Error) {
		return `${errorLike.name}: ${errorLike.message}`;
	}
	if (typeof errorLike === "string") {
		return errorLike;
	}
	try {
		return JSON.stringify(errorLike);
	} catch {
		return String(errorLike);
	}
}

export async function initializeAppLogger() {
	if (didInit) {
		return;
	}
	didInit = true;
	await info("frontend logger initialized");
}

export function logInfo(message: string) {
	void info(message);
}

export function logDebug(message: string) {
	void debug(message);
}

export function logWarn(message: string) {
	void warn(message);
}

export function logError(message: string, errorLike?: unknown) {
	const suffix =
		errorLike === undefined ? "" : ` | ${serializeError(errorLike)}`;
	void error(`${message}${suffix}`);
}
