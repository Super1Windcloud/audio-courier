import { invoke, isTauri } from "@tauri-apps/api/core";
import React from "react";
import ReactDOM from "react-dom/client";
import { initializeAppLogger, logError, logInfo } from "@/lib/logger.ts";
import useAppStateStore from "@/stores";
import {
	type ProviderEnvPresets,
	stripLlmApiKeyPresetsFromSettings,
} from "@/types/provider.ts";
import App from "./App.tsx";

const root = ReactDOM.createRoot(
	document.getElementById("root") as HTMLElement,
);

function logDevPromptEnv() {
	if (!import.meta.env.DEV) {
		return;
	}

	const vitePrompt = import.meta.env.VITE_PROMPT ?? "";
	const viteInterviewPrompt = import.meta.env.VITE_INTERVIEW_PROMPT ?? "";

	console.info("VITE_PROMPT:", vitePrompt);
	console.info("VITE_INTERVIEW_PROMPT:", viteInterviewPrompt);
	logInfo(`VITE_PROMPT: ${JSON.stringify(vitePrompt)}`);
	logInfo(`VITE_INTERVIEW_PROMPT: ${JSON.stringify(viteInterviewPrompt)}`);
}

async function bootstrap() {
	try {
		await initializeAppLogger();
	} catch (error) {
		console.error("initialize-app-logger-failed", error);
		logError("initialize-app-logger-failed", error);
	}

	logDevPromptEnv();

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
			const appState = useAppStateStore.getState();
			appState.updateEnvProviderPresets(presets);

			const sanitizedSettings = stripLlmApiKeyPresetsFromSettings(
				appState.llmProviderSettings,
				presets.llm,
			);
			if (
				JSON.stringify(sanitizedSettings) !==
				JSON.stringify(appState.llmProviderSettings)
			) {
				appState.updateLlmProviderSettings(sanitizedSettings);
			}
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
