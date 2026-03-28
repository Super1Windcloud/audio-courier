import { isTauri } from "@tauri-apps/api/core";
import { LazyStore } from "@tauri-apps/plugin-store";
import {
	createJSONStorage,
	type PersistStorage,
	type StateStorage,
} from "zustand/middleware";
import { logError } from "@/lib/logger.ts";

const APP_CONFIG_STORE_PATH = "app-config.json";
const LEGACY_UI_OPACITY_STORAGE_KEY = "audio-courier-ui-opacity";
const LEGACY_UI_TEXT_TONE_STORAGE_KEY = "audio-courier-ui-text-tone";

export const APP_CONFIG_STATE_KEY = "audio-courier-app-state";

const tauriAppConfigStore = new LazyStore(APP_CONFIG_STORE_PATH, {
	autoSave: 100,
	defaults: {},
});

const noopStorage: StateStorage = {
	getItem: () => null,
	setItem: () => {},
	removeItem: () => {},
};

const browserStorage: StateStorage = {
	getItem: (name) => {
		if (typeof window === "undefined") {
			return null;
		}

		return window.localStorage.getItem(name);
	},
	setItem: (name, value) => {
		if (typeof window === "undefined") {
			return;
		}

		window.localStorage.setItem(name, value);
	},
	removeItem: (name) => {
		if (typeof window === "undefined") {
			return;
		}

		window.localStorage.removeItem(name);
	},
};

const tauriStorage: StateStorage<Promise<void>> = {
	async getItem(name) {
		const value = await tauriAppConfigStore.get<string>(name);
		return value ?? null;
	},
	async setItem(name, value) {
		await tauriAppConfigStore.set(name, value);
	},
	async removeItem(name) {
		await tauriAppConfigStore.delete(name);
	},
};

function getAppConfigStorageBackend() {
	if (typeof window === "undefined") {
		return noopStorage;
	}

	return isTauri() ? tauriStorage : browserStorage;
}

export function createAppConfigStorage<S>(): PersistStorage<S> {
	const storage = createJSONStorage<S>(() => getAppConfigStorageBackend());
	if (!storage) {
		throw new Error("app config storage is unavailable");
	}

	return storage;
}

export function readLegacyUiConfigDefaults() {
	if (typeof window === "undefined") {
		return {};
	}

	const defaults: {
		uiOpacity?: number;
		uiTextTone?: "light" | "dark";
	} = {};

	try {
		const rawUiOpacity = window.localStorage.getItem(
			LEGACY_UI_OPACITY_STORAGE_KEY,
		);
		if (rawUiOpacity) {
			const parsedUiOpacity = Number(rawUiOpacity);
			if (!Number.isNaN(parsedUiOpacity)) {
				defaults.uiOpacity = parsedUiOpacity;
			}
		}

		const rawUiTextTone = window.localStorage.getItem(
			LEGACY_UI_TEXT_TONE_STORAGE_KEY,
		);
		if (rawUiTextTone === "light" || rawUiTextTone === "dark") {
			defaults.uiTextTone = rawUiTextTone;
		}
	} catch (error) {
		logError("read-legacy-ui-config-defaults failed", error);
	}

	return defaults;
}
