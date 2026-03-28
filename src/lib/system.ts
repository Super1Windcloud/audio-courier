import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
	isRegistered,
	register,
	unregister,
} from "@tauri-apps/plugin-global-shortcut";
import { toast } from "sonner";
import { logError, logInfo } from "@/lib/logger.ts";
import useAppStateStore from "@/stores";

function isWindows() {
	return navigator.userAgent.includes("Windows");
}

export async function showWindow() {
	if (!isWindows()) {
		return invoke<void>("show_window");
	}

	return new Promise<void>((resolve, reject) => {
		window.setTimeout(() => {
			requestAnimationFrame(() => {
				void invoke<void>("show_window").then(resolve).catch(reject);
			});
		}, 200);
	});
}

export async function toggleRecording() {
	const appState = useAppStateStore.getState();

	if (!appState.isRecording) {
		appState.updateIsRecording(true);
	} else {
		const startedAt = appState.recordingStartedAt ?? 0;
		if (startedAt && Date.now() - startedAt < 3000) {
			toast.warning("录音开始后需要等待 3 秒才能停止");
		} else {
			appState.updateIsRecording(false);
		}
	}
}

export async function registryGlobalShortCuts() {
	const combo = "CommandOrControl+Shift+`";
	if (await isRegistered(combo)) {
		await unregister(combo);
	}

	await register(combo, async (event) => {
		if (event.state === "Released") {
			const window = getCurrentWindow();

			if (await window.isVisible()) {
				await window.hide();
			} else {
				await showWindow();
			}
		}
	});

	const recordCombo = "Alt+Space";
	if (await isRegistered(recordCombo)) {
		await unregister(recordCombo);
	}

	await register(recordCombo, async (event) => {
		if (event.state === "Released") {
			await toggleRecording();
		}
	});

	const devtoolsCombos = ["CommandOrControl+F12"];
	for (const combo of devtoolsCombos) {
		try {
			if (await isRegistered(combo)) {
				await unregister(combo);
			}

			await register(combo, async (event) => {
				if (event.state !== "Released") {
					return;
				}
				try {
					await invoke("toggle_devtools");
					logInfo(`toggle-devtools via ${combo}`);
				} catch (error) {
					logError(`toggle-devtools failed via ${combo}`, error);
					toast.error(String(error));
				}
			});
		} catch (error) {
			logError(`register devtools shortcut failed for ${combo}`, error);
		}
	}
}
