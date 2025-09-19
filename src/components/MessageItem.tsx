import React from "react";
import { formatDistanceToNow } from "date-fns";
import { Check, CheckCheck, Clock } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Message } from "./ChatContainer";

interface MessageItemProps {
  message: Message;
}

export const MessageItem: React.FC<MessageItemProps> = ({ message }) => {
  const isUser = message.sender === "user";

  const getStatusIcon = () => {
    switch (message.status) {
      case "sending":
        return <Clock className="w-3 h-3 text-muted-foreground" />;
      case "sent":
        return <Check className="w-3 h-3 text-muted-foreground" />;
      case "delivered":
        return <CheckCheck className="w-3 h-3 text-muted-foreground" />;
      case "read":
        return <CheckCheck className="w-3 h-3 text-blue-500" />;
      default:
        return null;
    }
  };

  return (
    <div
      className={cn(
        "flex w-full animate-in slide-in-from-bottom-2 duration-300",
        isUser ? "justify-end" : "justify-start",
      )}
    >
      <div
        className={cn(
          "relative max-w-[70%] rounded-2xl px-4 py-2 shadow-sm",
          isUser
            ? "bg-blue-500 text-white rounded-br-md"
            : "bg-muted text-foreground rounded-bl-md",
        )}
      >
        <p className="text-sm leading-relaxed break-words">{message.text}</p>
        <div
          className={cn(
            "flex items-center justify-end space-x-1 mt-1",
            isUser ? "text-blue-100" : "text-muted-foreground",
          )}
        >
          <span className="text-xs">
            {formatDistanceToNow(message.timestamp, { addSuffix: true })}
          </span>
          {isUser && getStatusIcon()}
        </div>
      </div>
    </div>
  );
};
