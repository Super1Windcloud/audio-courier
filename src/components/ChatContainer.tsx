import React, { useState } from "react";
import { MessageList } from "./MessageList";
import { MessageInput } from "./MessageInput";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Card } from "@/components/ui/card";

export interface Message {
  id: string;
  text: string;
  timestamp: Date;
  sender: "user" | "contact";
  status?: "sending" | "sent" | "delivered" | "read";
}

const SIMULATED_REPLIES = [
  "That's interesting! Tell me more.",
  "I completely agree with you.",
  "Thanks for sharing that with me.",
  "How was your day?",
  "That sounds like a great idea!",
  "I'll think about it and get back to you.",
  "Nice! I'm happy to hear that.",
  "What do you think about this weather?",
  "Let's catch up soon!",
  "I was just thinking about you.",
];

export const ChatContainer: React.FC = () => {
  const [messages, setMessages] = useState<Message[]>([
    {
      id: "1",
      text: "Hey! How are you doing today?",
      timestamp: new Date(Date.now() - 3600000),
      sender: "contact",
      status: "read",
    },
    {
      id: "2",
      text: "I'm doing great! Just working on some exciting projects.",
      timestamp: new Date(Date.now() - 3500000),
      sender: "user",
      status: "read",
    },
    {
      id: "3",
      text: "That sounds awesome! What kind of projects?",
      timestamp: new Date(Date.now() - 3400000),
      sender: "contact",
      status: "read",
    },
  ]);

  const [isTyping, setIsTyping] = useState(false);

  const handleSendMessage = (text: string) => {
    const newMessage: Message = {
      id: Date.now().toString(),
      text,
      timestamp: new Date(),
      sender: "user",
      status: "sending",
    };

    setMessages((prev) => [...prev, newMessage]);

    // Simulate message delivery
    setTimeout(() => {
      setMessages((prev) =>
        prev.map((msg) =>
          msg.id === newMessage.id
            ? { ...msg, status: "delivered" as const }
            : msg,
        ),
      );
    }, 1000);

    // Simulate contact typing and reply
    setTimeout(() => {
      setIsTyping(true);
    }, 1500);

    setTimeout(() => {
      setIsTyping(false);
      const replyText =
        SIMULATED_REPLIES[Math.floor(Math.random() * SIMULATED_REPLIES.length)];
      const replyMessage: Message = {
        id: (Date.now() + 1).toString(),
        text: replyText,
        timestamp: new Date(),
        sender: "contact",
        status: "read",
      };
      setMessages((prev) => [...prev, replyMessage]);
    }, 3000);
  };

  return (
    <div className="flex flex-col h-screen max-w-4xl mx-auto">
      {/* Chat Header */}
      <Card className="flex-shrink-0 border-b rounded-none border-l-0 border-r-0 border-t-0">
        <div className="flex items-center p-4 space-x-3">
          <Avatar className="w-10 h-10">
            <AvatarImage
              src="https://images.pexels.com/photos/1239291/pexels-photo-1239291.jpeg?auto=compress&cs=tinysrgb&w=150"
              alt="Contact"
            />
            <AvatarFallback className="bg-blue-500 text-white">
              JD
            </AvatarFallback>
          </Avatar>
          <div className="flex-1">
            <h3 className="font-semibold text-foreground">John Doe</h3>
            <p className="text-sm text-muted-foreground">
              {isTyping ? "typing..." : "last seen recently"}
            </p>
          </div>
        </div>
      </Card>

      {/* Messages */}
      <div className="flex-1 overflow-hidden">
        <MessageList messages={messages} isTyping={isTyping} />
      </div>

      {/* Message Input */}
      <div className="flex-shrink-0 border-t bg-background">
        <MessageInput onSendMessage={handleSendMessage} />
      </div>
    </div>
  );
};
