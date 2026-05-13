import type React from "react";
import { cn } from "@/lib/utils";
import type { Message } from "./ChatContainer";
import { MarkdownMessage } from "./MarkdownMessage";

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
					"relative rounded-2xl px-4 py-2 shadow-sm backdrop-blur-md  bg-white/10 border border-white/10",
					isUser
						? "max-w-[70%] text-white rounded-br-md"
						: "w-full max-w-none text-white rounded-bl-md",
				)}
			>
				{isUser ? (
					<p className="text-sm leading-relaxed break-words">{message.text}</p>
				) : (
					<div className={"flex flex-col"}>
						<MarkdownMessage content={message.text} />
					</div>
				)}
			</div>
		</div>
	);
};
