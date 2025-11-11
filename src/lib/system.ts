import { getCurrentWindow } from "@tauri-apps/api/window";
import {
	isRegistered,
	register,
	unregister,
} from "@tauri-apps/plugin-global-shortcut";

export async function registryGlobalShortCuts() {
	const combo = "CommandOrControl+Alt+Q";
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
}
