import { check, type Update } from "@tauri-apps/plugin-updater";
import { toast } from "sonner";

function formatBytes(bytes?: number) {
	if (!bytes || Number.isNaN(bytes)) {
		return "";
	}

	return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function toErrorMessage(error: unknown) {
	if (error instanceof Error) {
		return error.message;
	}

	return String(error);
}

export async function checkAndInstallUpdate() {
	const update = await check();
	if (!update) {
		toast.success("当前已是最新版本");
		return;
	}

	toast.message(`发现新版本 ${update.version}`, {
		description: update.body ?? "开始下载并安装更新包",
	});

	await downloadAndInstallUpdate(update);
}

async function downloadAndInstallUpdate(update: Update) {
	let totalBytes = 0;
	let downloadedBytes = 0;

	await update.downloadAndInstall((event) => {
		if (event.event === "Started") {
			totalBytes = event.data.contentLength ?? 0;
			toast.message("开始下载更新", {
				description: totalBytes
					? `更新包大小约 ${formatBytes(totalBytes)}`
					: "正在获取更新包大小",
			});
			return;
		}

		if (event.event === "Progress") {
			downloadedBytes += event.data.chunkLength;
			if (totalBytes > 0 && downloadedBytes >= totalBytes) {
				toast.message("更新包下载完成", {
					description: "正在启动安装流程",
				});
			}
		}
	});

	toast.success("更新已交给安装器处理", {
		description: "如果应用退出或弹出安装窗口，属于正常行为。",
	});
}

export async function runUpdater() {
	try {
		await checkAndInstallUpdate();
	} catch (error) {
		toast.error("更新失败", {
			description: toErrorMessage(error),
		});
	}
}
