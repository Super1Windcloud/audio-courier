import { ModelOption } from "@/types/llm.ts";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function llmChatStreamOutput(
  question: string,
  currentModel: ModelOption,
  renderCallback: (chunk: string) => void,
) {
  await invoke(currentModel, { question });
  let result = "";
    await listen<string>("llm_stream", (event) => {
    console.log("llm_stream", event.payload);
    result += event.payload;

    renderCallback(result);
  });
}
