import React, { useEffect, useRef, useState } from "react";
import TitleBar from "@/components/TitleBar.tsx";
import { llmInterviewChatStreamOutput } from "@/lib/llm.ts";
import useAppStateStore from "@/stores";
import { MessageInput } from "./MessageInput";
import { MessageList } from "./MessageList";

export interface Message {
  id: number;
  text: string;
  sender: "user" | "robot";
}

export const ChatContainer: React.FC = () => {
  const didRun = useRef(false);
  const [messages, setMessages] = useState<Message[]>([
    {
      id: 0,
      text: "你好,请开始你的语音对话",
      sender: "robot",
    },
  ]);
  useEffect(() => {
    if (didRun.current) return;
    didRun.current = true;
    if (import.meta.env.VITE_INIT_MESSAGE) {
      handleSendMessage(import.meta.env.VITE_INIT_MESSAGE, true);
    }
  }, []);

  const [isTyping, setIsTyping] = useState(false);
  const appState = useAppStateStore();

  const handleSendMessage = async (text: string, introduceSelf?: boolean) => {
    setMessages((prev) => {
      const userMessage: Message = {
        id: prev.length,
        text,
        sender: "user",
      };
      return [...prev, userMessage];
    });
    setIsTyping(true);

    await llmInterviewChatStreamOutput(
      text,
      introduceSelf
        ? import.meta.env.VITE_INTERVIEW_PROMPT
        : appState.llmPrompt,
      appState.currentSelectedModel,
      (content) => {
        setIsTyping(false);

        setMessages((prev) => {
          return prev.length > 0 && prev[prev.length - 1].sender === "robot"
            ? [
                ...prev.slice(0, -1), // 去掉最后一个
                { ...prev[prev.length - 1], text: content }, // 替换最后一个
              ]
            : [...prev, { text: content, sender: "robot", id: prev.length }];
        });
      },
    );
  };

  const handleClearConversation = () => {
    setMessages([
      {
        id: 0,
        text: "你好,请开始你的语音对话",
        sender: "robot",
      },
    ]);
    setIsTyping(false);
  };

  return (
    <div className="flex flex-col h-screen w-screen justify-center">
      <div className="flex-shrink-0">
        <TitleBar />
      </div>
      <div
        className="flex-1 overflow-auto max-w-5xl w-full  self-center"
        style={{
          overflow: "auto",
          scrollBehavior: "smooth",
          scrollbarWidth: "none",
        }}
      >
        <MessageList messages={messages} isTyping={isTyping} />
      </div>

      <div className="flex-shrink-0 bg-transparent  w-full  max-w-5xl self-center">
        <MessageInput
          onSendMessage={handleSendMessage}
          onClearConversation={handleClearConversation}
          setIsTyping={setIsTyping}
          setMessages={setMessages}
        />
      </div>
    </div>
  );
};
