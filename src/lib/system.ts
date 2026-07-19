import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
	isRegistered,
	register,
	unregister,
} from "@tauri-apps/plugin-global-shortcut";
import { toast } from "sonner";
import { logError } from "@/lib/logger.ts";
import { setRecordingStateImmediately } from "@/lib/recordingState.ts";
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
		setRecordingStateImmediately(true);
	} else {
		const startedAt = appState.recordingStartedAt ?? 0;
		if (startedAt && Date.now() - startedAt < 3000) {
			toast.warning("录音开始后需要等待 3 秒才能停止");
		} else {
			setRecordingStateImmediately(false);
		}
	}
}

export async function registryGlobalShortCuts() {
	const shortcuts = [
		{
			combo: "CommandOrControl+Shift+`",
			handler: async () => {
				const window = getCurrentWindow();
				if (await window.isVisible()) {
					await window.hide();
				} else {
					await showWindow();
				}
			},
		},
		{ combo: "Alt+Space", handler: toggleRecording },
		{
			combo: "Shift+Enter",
			handler: async () => emit("global_send_shortcut"),
		},
		{
			combo: "CommandOrControl+F12",
			handler: async () => {
				await invoke("toggle_devtools");
			},
		},
	];

	for (const { combo, handler } of shortcuts) {
		try {
			if (await isRegistered(combo)) {
				await unregister(combo);
			}

			await register(combo, async (event) => {
				if (event.state !== "Released") {
					return;
				}
				try {
					await handler();
				} catch (error) {
					logError(`global shortcut handler failed for ${combo}`, error);
					toast.error(String(error));
				}
			});
		} catch (error) {
			logError(`register global shortcut failed for ${combo}`, error);
			window.alert(
				`快捷键 ${combo} 存在冲突，请关闭占用该快捷键的程序后重试。`,
			);
		}
	}
}
