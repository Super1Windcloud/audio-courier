import { getCurrentWindow } from "@tauri-apps/api/window";
import {
	isRegistered,
	register,
	unregister,
} from "@tauri-apps/plugin-global-shortcut";
import { toast } from "sonner";
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
}
