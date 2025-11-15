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
  console.log("llmPrompt", llmPrompt);
  let _llmPrompt = "回答问题";
  let result = "";
  const unlisten = await listen<string>("llm_stream", (event) => {
    result += event.payload;
    renderCallback(result);
  });
  invoke(currentModel,  {
    flowArgs: {
      question,
      llmPrompt: _llmPrompt,
    },
  })
    .then((final: unknown) => {
      console.warn(question === "go" ? "go result :" : "rust result", final);
    })
    .catch((err) => {
      console.error("invoke llmModel", err);
    })
    .finally(() => {
      unlisten();
    });
}
