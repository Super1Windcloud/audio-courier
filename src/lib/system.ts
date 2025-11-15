import { getCurrentWindow } from "@tauri-apps/api/window";
import {
	isRegistered,
	register,
	unregister,
} from "@tauri-apps/plugin-global-shortcut";
import useAppStateStore from "@/stores";

// 在非TSX组件上下文, 不能够使用React Hook订阅字段, 只能getState读取,或者使用subscribe()订阅
export async function toggleRecording() {
	const appState = useAppStateStore.getState();

	if (!appState.isRecording) {
		appState.updateIsRecording(true);
	} else {
		appState.updateIsRecording(false);
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
