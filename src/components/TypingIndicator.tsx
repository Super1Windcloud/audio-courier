import React from "react";

export const TypingIndicator: React.FC = () => {
  return (
    <div className="flex items-end space-x-2 animate-in slide-in-from-bottom-2 duration-300">
      <div className="bg-gray-600 rounded-2xl rounded-bl-md px-4 py-2">
        <div className="flex space-x-1">
          <div
            className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce"
            style={{ animationDelay: "0ms" }}
          ></div>
          <div
            className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce"
            style={{ animationDelay: "150ms" }}
          ></div>
          <div
            className="w-2 h-2 bg-muted-foreground rounded-full animate-bounce"
            style={{ animationDelay: "300ms" }}
          ></div>
        </div>
      </div>
    </div>
  );
};
