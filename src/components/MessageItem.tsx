import React from "react";
import { cn } from "@/lib/utils";
import type { Message } from "./ChatContainer";

interface MessageItemProps {
	message: Message;
}

export const MessageItem: React.FC<MessageItemProps> = ({ message }) => {
	const isUser = message.sender === "user";

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
						? "text-white rounded-br-md  border-gray-200"
						: "text-white bg-gray-600 rounded-bl-md",
				)}
			>
				{isUser ? (
					<p className="text-sm leading-relaxed break-words">{message.text}</p>
				) : (
					<div className={"flex flex-col"}>
						<p className="text-sm leading-relaxed break-words">
							{message.text}
						</p>
						<button
							className="bg-blue-100/80 ml-auto  text-xs
             hover:bg-blue-200/80  text-gray-600
               font-medium py-1 px-2 rounded-full border
               hover:shadow-xl hover:shadow-blue-300/60
                transition-all duration-300 active:scale-95 relative overflow-hidden"
							style={{
								borderRadius: 30,
								boxShadow:
									"inset 3px 4px 8px -5px #00000040, 3px 2px 3.5px 0px #1B1F3340",
							}}
						>
							开始对话
						</button>
					</div>
				)}
			</div>
		</div>
	);
};
