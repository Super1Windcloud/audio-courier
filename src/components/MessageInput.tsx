import React, { useEffect, useState } from "react";
import { SendHorizontal, Mic, Trash2 } from "lucide-react";
import { Input } from "@/components/ui/input";

import { Message } from "@/components/ChatContainer.tsx";
import { startAudioRecognition, stopAudioRecognition } from "@/lib/audio.ts";
import useAppStateStore from "@/stores";
import { MoreMenu } from "@/components/MoreMenu.tsx";
import { clearVoskAcceptBuffer } from "@/lib/cpal.ts";

interface MessageInputProps {
  onSendMessage: (text: string) => void;
  onClearConversation: () => void;
  setMessages: React.Dispatch<React.SetStateAction<Message[]>>;
  setIsTyping: (record: boolean) => void;
}

export const MessageInput: React.FC<MessageInputProps> = ({
  onSendMessage,
  onClearConversation,
  setIsTyping,
}) => {
  const [isRecording, setIsRecording] = useState(false);
  const [inputText, setInputText] = useState("");
  const appState = useAppStateStore();

  const handleSend = () => {
    if (inputText.trim()) {
      appState.updateQuestion(inputText.trim());
      onSendMessage(inputText.trim());
      setInputText("");
      if (isRecording) {
        clearVoskAcceptBuffer().then(() => {
        });
      }
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const toggleRecording = async () => {
    if (!isRecording) {
      setIsRecording(true);
      await startAudioRecognition(
        setInputText,
        appState.currentAudioChannel,
        appState.captureInterval,
      );
    } else {
      setIsRecording(false);
      await stopAudioRecognition(appState.currentAudioChannel);
    }
  };

  useEffect(() => {
    setIsTyping(isRecording);
  }, [isRecording]);

  const handleClearConversation = async () => {
    setInputText("");
    setIsRecording(false);
    if (isRecording) {
      await stopAudioRecognition(appState.currentAudioChannel);
    }
    onClearConversation();
  };

  return (
    <div
      className="p-4
      relative   px-4 py-2 shadow-sm
      backdrop-blur-xl  bg-white/10 border border-white/10"
    >
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
