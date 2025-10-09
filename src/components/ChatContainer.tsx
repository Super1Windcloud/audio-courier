import React, { useState } from "react";
import { MessageList } from "./MessageList";
import { MessageInput } from "./MessageInput";
import { llmChatStreamOutput } from "@/lib/llm.ts";
import useAppStateStore from "@/stores";

export interface Message {
  id: number;
  text: string;
  sender: "user" | "robot";
}

export const ChatContainer: React.FC = () => {
  // 存储到本地, 消息历史
  const [messages, setMessages] = useState<Message[]>([
    {
      id: 0,
      text: import.meta.env.VITE_INIT_MESSAGE || "你好,请开始你的语音对话",
      sender: "robot",
    },
  ]);

  const [isTyping, setIsTyping] = useState(false);
  const appState = useAppStateStore();

  const handleSendMessage = async (text: string) => {
    setMessages((prev) => {
      const userMessage: Message = {
        id: prev.length,
        text,
        sender: "user",
      };
      return [...prev, userMessage];
    });
    setIsTyping(true);

    await llmChatStreamOutput(
      text,
      appState.llmPrompt,
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
        text: import.meta.env.VITE_INIT_MESSAGE || "你好,请开始你的语音对话",
        sender: "robot",
      },
    ]);
    setIsTyping(false);
  };
  //记录最后一次识别的时间
  let lastRecognizedAt = 0;

  const handleMessageCapture = (content: string) => {
    setIsTyping(false);

    setMessages((prev) => {
      const now = Date.now();
      const timeDiff = now - lastRecognizedAt;

      lastRecognizedAt = now; // 更新最新识别时间

      if (
        prev.length > 0 &&
        prev[prev.length - 1].sender === "robot" &&
        timeDiff < 3000 // 3s 内
      ) {
        // 继续追加到最后一条
        return [
          ...prev.slice(0, -1),
          {
            ...prev[prev.length - 1],
            text: prev[prev.length - 1].text + content,
          },
        ];
      } else {
        // 新增一条
        return [...prev, { text: content, sender: "robot", id: prev.length }];
      }
    });
  };

  return (
    <div className="flex flex-col h-screen max-w-4xl mx-auto">
      <div
        className="flex-1 overflow-auto w-full"
        style={{
          overflow: "auto",
          scrollBehavior: "smooth",
          scrollbarWidth: "none",
        }}
      >
        <MessageList messages={messages} isTyping={isTyping} />
      </div>

      <div className="flex-shrink-0 bg-transparent">
        <MessageInput
          onSendMessage={handleSendMessage}
          onClearConversation={handleClearConversation}
          onMessageCapture={handleMessageCapture}
          setIsTyping={setIsTyping}
          setMessages={setMessages}
        />
      </div>
    </div>
  );
};
