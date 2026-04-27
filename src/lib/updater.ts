import { isTauri } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { toast } from "sonner";

export const OPEN_UPDATER_DIALOG_EVENT = "audio-courier:open-updater-dialog";

export function formatBytes(bytes?: number) {
	if (!bytes || Number.isNaN(bytes)) {
		return "";
	}

	return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function toErrorMessage(error: unknown) {
	if (error instanceof Error) {
		return error.message;
	}

	return String(error);
}

export function isUpdaterSupported() {
	return isTauri();
}

export async function checkForUpdate() {
	if (!isUpdaterSupported()) {
		return null;
	}

	return await check();
}

export async function downloadAndInstallUpdate(
	update: Update,
	onProgress?: (event: {
		stage: "started" | "progress" | "finished";
		totalBytes: number;
		downloadedBytes: number;
	}) => void,
) {
	if (!isUpdaterSupported()) {
		throw new Error("当前仅 Tauri 桌面端支持应用更新");
	}

	let totalBytes = 0;
	let downloadedBytes = 0;

	await update.downloadAndInstall((event) => {
		if (event.event === "Started") {
			totalBytes = event.data.contentLength ?? 0;
			onProgress?.({
				stage: "started",
				totalBytes,
				downloadedBytes,
			});
			toast.message("开始下载更新", {
				description: totalBytes
					? `更新包大小约 ${formatBytes(totalBytes)}`
					: "正在获取更新包大小",
			});
			return;
		}

		if (event.event === "Progress") {
			downloadedBytes += event.data.chunkLength;
			onProgress?.({
				stage: "progress",
				totalBytes,
				downloadedBytes,
			});
			if (totalBytes > 0 && downloadedBytes >= totalBytes) {
				onProgress?.({
					stage: "finished",
					totalBytes,
					downloadedBytes,
				});
				toast.message("更新包下载完成", {
					description: "正在启动安装流程",
				});
			}
		}
	});

	toast.success("更新安装完成", {
		description: "应用即将重启。",
	});

	await relaunch();
}

export async function runUpdater() {
	if (!isUpdaterSupported()) {
		toast.message("当前运行在浏览器预览模式", {
			description: "检查更新仅在 Tauri 桌面端可用。",
		});
		return;
	}

	try {
		window.dispatchEvent(new CustomEvent(OPEN_UPDATER_DIALOG_EVENT));
	} catch (error) {
		toast.error("更新失败", {
			description: toErrorMessage(error),
		});
	}
}
