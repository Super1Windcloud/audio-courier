import { isTauri } from "@tauri-apps/api/core";
import { LazyStore } from "@tauri-apps/plugin-store";
import type { PersistStorage, StorageValue } from "zustand/middleware";
import { logError } from "@/lib/logger.ts";

const APP_CONFIG_STORE_PATH = "audio-courier.settings.json";
const LEGACY_UI_OPACITY_STORAGE_KEY = "audio-courier-ui-opacity";
const LEGACY_UI_TEXT_TONE_STORAGE_KEY = "audio-courier-ui-text-tone";

export const APP_CONFIG_STATE_KEY = "audio-courier-app-state";

const tauriAppConfigStore = new LazyStore(APP_CONFIG_STORE_PATH, {
	autoSave: 100,
	defaults: {},
});

function parsePersistedStorageValue<S>(
	value: StorageValue<S> | string | null | undefined,
) {
	if (!value) {
		return null;
	}

	if (typeof value === "string") {
		return JSON.parse(value) as StorageValue<S>;
	}

	return value;
}

function createNoopStorage<S>(): PersistStorage<S> {
	return {
		getItem: () => null,
		setItem: () => {},
		removeItem: () => {},
	};
}

function createBrowserStorage<S>(): PersistStorage<S> {
	return {
		getItem: (name) => {
			if (typeof window === "undefined") {
				return null;
			}

			const rawValue = window.localStorage.getItem(name);
			if (!rawValue) {
				return null;
			}

			return JSON.parse(rawValue) as StorageValue<S>;
		},
		setItem: (name, value) => {
			if (typeof window === "undefined") {
				return;
			}

			window.localStorage.setItem(name, JSON.stringify(value));
		},
		removeItem: (name) => {
			if (typeof window === "undefined") {
				return;
			}

			window.localStorage.removeItem(name);
		},
	};
}

function createTauriStorage<S>(): PersistStorage<S, Promise<void>> {
	return {
		async getItem(name) {
			try {
				const rawValue = await tauriAppConfigStore.get<
					StorageValue<S> | string
				>(name);
				const parsedValue = parsePersistedStorageValue(rawValue);

				// Migrate old stringified payloads to plain JSON objects on first read.
				if (typeof rawValue === "string" && parsedValue) {
					await tauriAppConfigStore.set(name, parsedValue);
				}

				return parsedValue;
			} catch (error) {
				logError("read-tauri-app-config failed", error);
				return null;
			}
		},
		async setItem(name, value) {
			try {
				await tauriAppConfigStore.set(name, value);
			} catch (error) {
				logError("write-tauri-app-config failed", error);
			}
		},
		async removeItem(name) {
			try {
				await tauriAppConfigStore.delete(name);
			} catch (error) {
				logError("remove-tauri-app-config failed", error);
			}
		},
	};
}

export function createAppConfigStorage<S>(): PersistStorage<S> {
	if (typeof window === "undefined") {
		return createNoopStorage<S>();
	}

	return isTauri() ? createTauriStorage<S>() : createBrowserStorage<S>();
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
