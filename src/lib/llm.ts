import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toast } from "sonner";
import { logError, logInfo } from "@/lib/logger.ts";
import type { ModelOption } from "@/types/llm.ts";

export async function llmInterviewChatStreamOutput(
	question: string,
	llmPrompt: string,
	currentModel: ModelOption,
	renderCallback: (chunk: string) => void,
) {
	logInfo(`llm-start model=${currentModel} questionLength=${question.length}`);
	let result = "";

	const requestId = Math.random().toString(36).substring(2, 15);
	const eventName = `llm_stream_${requestId}`;

	const unlisten = await listen<string>(eventName, (event) => {
		result += event.payload;
		renderCallback(result);
	});

	invoke(currentModel, {
		flowArgs: {
			question,
			llmPrompt,
			requestId,
		},
	})
		.then((_final: unknown) => {
			// console.warn(question === "go" ? "go result :" : "rust result", _final);
		})
		.catch((err) => {
			console.error("invoke llmModel Error", err);
			logError("invoke llmModel Error", err);
			toast.error(`invoke llm err ${err}`);
		})
		.finally(() => {
			unlisten();
		});
}
