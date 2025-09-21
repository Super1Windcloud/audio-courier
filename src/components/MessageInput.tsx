import React, { useEffect, useState } from "react";
import { MoreVertical, SendHorizontal, Mic, Trash2 } from "lucide-react";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Message } from "@/components/ChatContainer.tsx";
import { startAudioRecognition, stopAudioRecognition } from "@/lib/audio.ts";
import { MODEL_OPTIONS, ModelOption } from "@/types/llm.ts";
import useAppStateStore from "@/stores";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { Button } from "@/components/ui/button.tsx";

export function MoreMenu() {
  const [currentModel, setCurrentModel] = useState<ModelOption>("siliconflow");
  const appState = useAppStateStore();
  const [audioChannels, setAudioChannels] = useState<string[]>([]);
  const [isDialogOpen, setIsDialogOpen] = useState(false);

  useEffect(() => {
    invoke("get_audio_stream_devices_name").then((result) => {
      if (typeof result === "object" && Array.isArray(result)) {
        console.log("audio devices ", result);
        setAudioChannels(result);
        appState.updateCurrentAudioChannel(result[0]);
      } else {
        toast.error("No audio streams found");
      }
    });
  }, []);

  useEffect(() => {
    appState.updateCurrentSelectedModel(currentModel);
    console.log(currentModel);
  }, [currentModel]);

  const handleSubmit = () => {
    setIsDialogOpen(false);
  };

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <MoreVertical className="text-gray-400 cursor-pointer bg-transparent" />
        </DropdownMenuTrigger>
        <DropdownMenuContent
          align="end"
          className="w-40 bg-gray-600 text-white border-0"
        >
          <DropdownMenuItem
            onClick={() => setIsDialogOpen(true)}
            className="data-[highlighted]:bg-gray-500"
          >
            提示词
          </DropdownMenuItem>
          <DropdownMenuSub>
            <DropdownMenuSubTrigger
              className="
             bg-gray-600 text-white
            data-[highlighted]:bg-gray-500
            data-[state=open]:bg-gray-500"
            >
              {" "}
              大模型
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className="w-48 bg-gray-600 text-white border-0">
              {MODEL_OPTIONS.map((model) => (
                <DropdownMenuItem
                  key={model}
                  className={`data-[highlighted]:bg-gray-500 ${
                    currentModel === model ? "font-bold" : ""
                  }`}
                  onClick={() => setCurrentModel(model)}
                >
                  {model}
                  {currentModel === model && (
                    <span className="ml-2 text-green-400">✔</span>
                  )}
                </DropdownMenuItem>
              ))}
            </DropdownMenuSubContent>
          </DropdownMenuSub>
          <DropdownMenuSub>
            <DropdownMenuSubTrigger
              className="
             bg-gray-600 text-white
            data-[highlighted]:bg-gray-500
            data-[state=open]:bg-gray-500"
            >
              {" "}
              选择音频通道
            </DropdownMenuSubTrigger>
            <DropdownMenuSubContent className="w-48 bg-gray-600 text-white border-0">
              {audioChannels.map((devices) => (
                <DropdownMenuItem
                  key={devices}
                  className={`data-[highlighted]:bg-gray-500 ${
                    appState.currentAudioChannel === devices ? "font-bold" : ""
                  }`}
                  onClick={() => appState.updateCurrentAudioChannel(devices)}
                >
                  {devices}
                  {appState.currentAudioChannel === devices && (
                    <span className="ml-2 text-green-400">✔</span>
                  )}
                </DropdownMenuItem>
              ))}
            </DropdownMenuSubContent>
          </DropdownMenuSub>
        </DropdownMenuContent>
      </DropdownMenu>
      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent className="sm:max-w-[400px] bg-pink-200">
          <DialogHeader>
            <DialogTitle>输入提示词</DialogTitle>
          </DialogHeader>

          <Input
            value={appState.llmPrompt}
            onChange={(e) => appState.updateLLMPrompt(e.target.value)}
            placeholder="请输入提示词..."
            className="mt-2 w-full"
          />

          <DialogFooter className={"bg-transparent"}>
            <Button type={"submit"} variant={"ghost"} onClick={handleSubmit}>
              提交
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}

interface MessageInputProps {
  onSendMessage: (text: string) => void;
  onClearConversation?: () => void;
  onMessageCapture: (message: string, replyId: string) => void;
  setMessages: React.Dispatch<React.SetStateAction<Message[]>>;
  setIsTyping: (record: boolean) => void;
}

export const MessageInput: React.FC<MessageInputProps> = ({
  onSendMessage,
  onClearConversation,
  onMessageCapture,
  setMessages,
  setIsTyping,
}) => {
  const [isRecording, setIsRecording] = useState(false);
  const [inputText, setInputText] = useState("");
  const appState = useAppStateStore();
  const handleSend = () => {
    if (inputText.trim()) {
      onSendMessage(inputText.trim());
      appState.updateQuestion(inputText.trim());
      setInputText("");
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const toggleRecording = () => {
    if (!isRecording) {
      setIsRecording(true);
      startAudioRecognition(onMessageCapture, setMessages);
    } else {
      setIsRecording(false);
      stopAudioRecognition();
    }
  };

  useEffect(() => {
    setIsTyping(isRecording);
  }, [isRecording, setIsTyping]);
  const handleClearConversation = () => {
    setInputText("");
    onClearConversation?.();
  };

  return (
    <div className="p-4">
      <div className="flex border-none items-center space-x-2">
        <Input
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
          onKeyDown={handleKeyPress}
          placeholder="输入消息..."
          className="flex-1 text-white border-none focus-visible:ring-0   placeholder:text-gray-300 focus-visible:ring-offset-0"
        />

        <span title={isRecording ? "停止语音" : "开始语音"}>
          <Mic
            onClick={toggleRecording}
            className={`cursor-pointer ${isRecording ? "text-red-500" : "text-gray-400"}`}
          />
        </span>

        <span title="清空会话">
          <Trash2
            onClick={handleClearConversation}
            className="cursor-pointer text-gray-400"
          />
        </span>

        <span title="发送消息">
          <SendHorizontal
            onClick={handleSend}
            className="text-gray-400 cursor-pointer"
          />
        </span>

        <span title="更多选项">
          <MoreMenu />
        </span>
      </div>
    </div>
  );
};
