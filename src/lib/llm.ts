import { ModelOption } from "@/types/llm.ts";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export async function llmChatStreamOutput(
  question: string,
  _llmPrompt: string,
  currentModel: ModelOption,
  renderCallback: (chunk: string) => void,
) {
  console.log("currentModel", currentModel);
  console.log("currentQuestion", question);

  invoke(currentModel, {
    flowArgs: {
      question,
      llmPrompt: "你是代码助手,请回答我的问题",
    },
  });
  let result = "";
  await listen<string>("llm_stream", (event) => {
    result += event.payload;
    renderCallback(result);
  });
}
