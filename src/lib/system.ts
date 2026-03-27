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
				await window.show();
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

	const devtoolsCombos = ["F12", "CommandOrControl+Alt+I"];
	for (const combo of devtoolsCombos) {
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
	}
}
