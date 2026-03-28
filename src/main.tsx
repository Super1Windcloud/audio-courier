import { invoke, isTauri } from "@tauri-apps/api/core";
import React from "react";
import ReactDOM from "react-dom/client";
import { initializeAppLogger, logError } from "@/lib/logger.ts";
import useAppStateStore from "@/stores";
import type { ProviderEnvPresets } from "@/types/provider.ts";
import App from "./App.tsx";

const root = ReactDOM.createRoot(
	document.getElementById("root") as HTMLElement,
);

async function bootstrap() {
	try {
		await initializeAppLogger();
	} catch (error) {
		console.error("initialize-app-logger-failed", error);
		logError("initialize-app-logger-failed", error);
	}

	try {
		await useAppStateStore.persist.rehydrate();
	} catch (error) {
		console.error("rehydrate-app-config-failed", error);
		logError("rehydrate-app-config-failed", error);
	}

	if (isTauri()) {
		try {
			const presets = await invoke<ProviderEnvPresets>(
				"get_provider_env_presets",
			);
			useAppStateStore.getState().updateEnvProviderPresets(presets);
		} catch (error) {
			console.error("sync-provider-env-presets-failed", error);
			logError("sync-provider-env-presets-failed", error);
		}
	}

	root.render(
		<React.StrictMode>
			<App />
		</React.StrictMode>,
	);
}

void bootstrap();
