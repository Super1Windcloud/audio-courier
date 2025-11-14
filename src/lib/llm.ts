import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ModelOption } from "@/types/llm.ts";

export async function llmInterviewChatStreamOutput(
  question: string,
  llmPrompt: string,
  currentModel: ModelOption,
  renderCallback: (chunk: string) => void,
) {
  console.log("currentModel", currentModel);
  console.log("currentQuestion", question);
  let result = "";
  const unlisten = await listen<string>("llm_stream", (event) => {
    result += event.payload;
    renderCallback(result);
  });
  invoke(currentModel, {
    flowArgs: {
      question,
      llmPrompt,
    },
  })
    .catch((err) => {
      console.error("invoke llmModel", err);
    })
    .finally(() => {
      unlisten();
    });
}
