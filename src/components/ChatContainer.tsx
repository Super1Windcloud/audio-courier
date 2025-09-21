import React, { useState } from "react";
import { MessageList } from "./MessageList";
import { MessageInput } from "./MessageInput";
import { llmChatStreamOutput } from "@/lib/llm.ts";
import useAppStateStore from "@/stores";

export interface Message {
  id: string;
  text: string;
  timestamp: Date;
  sender: "user" | "robot";
}

export const ChatContainer: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([
    {
      id: Date.now().toString(),
      text: "你好,请开始你的语音对话",
      timestamp: new Date(),
      sender: "robot",
    },
  ]);

  const [isTyping, setIsTyping] = useState(false);
  const appState = useAppStateStore();

  const handleSendMessage = (text: string) => {
    const userMessage: Message = {
      id: Date.now().toString(),
      text,
      timestamp: new Date(),
      sender: "user",
    };

    setMessages((prev) => [...prev, userMessage]);
    setIsTyping(true);

    const contactMessageId = (Date.now() + 1).toString();
    const contactMessage: Message = {
      id: contactMessageId,
      text: "",
      timestamp: new Date(),
      sender: "robot",
    };
    setMessages((prev) => [...prev, contactMessage]);

    llmChatStreamOutput(
      appState.currentQuestion,
      appState.currentSelectedModel,
      (content) => {
        setIsTyping(false);
        // 更新 robot 消息的内容
        setMessages((prev) =>
          prev.map((msg) =>
            msg.id === contactMessageId
              ? {
                  ...msg,
                  text: msg.text + content,
                }
              : msg,
          ),
        );
      },
    );
  };

  const handleClearConversation = () => {
    setMessages([
      {
        id: Date.now().toString(),
        text: "你好,请开始你的语音对话",
        timestamp: new Date(),
        sender: "robot",
      },
    ]);
    setIsTyping(false);
  };

  const handleMessageCapture = (message: string, replyId: string) => {
    setIsTyping(false);
    setMessages((prev) =>
      prev.map((msg) => (msg.id === replyId ? { ...msg, text: message } : msg)),
    );
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

      <div className="flex-shrink-0 bg-gray-600">
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
