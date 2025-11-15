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
  // 用 ref 存消息，避免 React 状态更新导致未更新完成的旧的状态丢失
  const messagesRef = useRef<Message[]>([
    {
      id: 0,
      text: "你好,请开始你的语音对话",
      sender: "robot",
    },
  ]);

  const [messages, setMessages] = useState<Message[]>(messagesRef.current);
  useEffect(() => {
    if (didRun.current) return;
    didRun.current = true;
    if (import.meta.env.VITE_INIT_MESSAGE) {
      handleSendMessage(import.meta.env.VITE_INIT_MESSAGE, true).then();
    }
  }, []);

  const [isTyping, setIsTyping] = useState(false);
  const llmPromptStore = useAppStateStore((state) => state.llmPrompt);
  const currentSelectedModel = useAppStateStore(
    (state) => state.currentSelectedModel,
  );

  const handleSendMessage = async (text: string, introduceSelf?: boolean) => {
    const userMsg: Message = {
      id: messagesRef.current.length,
      text,
      sender: "user",
    };
    messagesRef.current.push(userMsg);

    // 添加机器人占位
    const botMsg: Message = {
      id: messagesRef.current.length,
      text: "",
      sender: "robot",
    };
    messagesRef.current.push(botMsg);

    setMessages([...messagesRef.current]);

    const thisBotId = botMsg.id; // ← 记录本轮机器人消息 ID

    setIsTyping(true);

    await llmInterviewChatStreamOutput(
      text,
      introduceSelf ? import.meta.env.VITE_INTERVIEW_PROMPT : llmPromptStore,
      currentSelectedModel,
      (content) => {
        setIsTyping(false);
        updateSpecificBotMessage(thisBotId, content); // ← 更新特定机器人消息
      },
    );
  };

  function updateSpecificBotMessage(id: number, content: string) {
    const msgs = messagesRef.current;
    const idx = msgs.findIndex((msg) => msg.id === id);

    if (idx === -1) return;

    msgs[idx] = {
      ...msgs[idx],
      text: content,
    };

    setMessages([...msgs]);
  }

  const handleClearConversation = () => {
    messagesRef.current = [
      {
        id: 0,
        text: "你好,请开始你的语音对话",
        sender: "robot",
      },
    ];
    setMessages([...messagesRef.current]);
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
