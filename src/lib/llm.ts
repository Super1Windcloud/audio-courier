import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { logError, logInfo, serializeError } from "@/lib/logger.ts";
import useAppStateStore from "@/stores";
import type { ModelOption } from "@/types/llm.ts";

interface LlmResolvedModelEvent {
	attempt: number;
	provider: string;
	model: string;
}

export async function llmInterviewChatStreamOutput(
	question: string,
	llmPrompt: string,
	currentModel: ModelOption,
	renderCallback: (chunk: string) => void,
) {
	let result = "";

	const requestId = Math.random().toString(36).substring(2, 15);
	const eventName = `llm_stream_${requestId}`;
	const metaEventName = `llm_meta_${requestId}`;
	const llmProviderSettings = useAppStateStore.getState().llmProviderSettings;

	const unlisten = await listen<string>(eventName, (event) => {
		result += event.payload;
		renderCallback(result);
	});
	const unlistenMeta = await listen<LlmResolvedModelEvent>(
		metaEventName,
		(event) => {
			const { attempt, provider, model } = event.payload;
			logInfo(
				`llm-attempt attempt=${attempt} provider=${provider} model=${model} questionLength=${question.length}`,
			);
		},
	);

	try {
		await invoke("chat_with_llm_provider", {
			provider: currentModel,
			flowArgs: {
				question,
				llmPrompt,
				requestId,
			},
			runtimeConfig: llmProviderSettings,
		});
	} catch (err) {
		const errorText = serializeError(err);
		console.error(`invoke llmModel Error model=${currentModel}`, err);
		logError(`invoke llmModel Error model=${currentModel}`, err);
		toast.error(`invoke llm err model=${currentModel} ${errorText}`);
		throw err;
	} finally {
		unlisten();
		unlistenMeta();
	}
}
