import React from "react";
import { useNavigate } from "react-router-dom";
import { cn } from "@/lib/utils";
import type { Message } from "./ChatContainer";

interface MessageItemProps {
	message: Message;
}

export const MessageItem: React.FC<MessageItemProps> = ({ message }) => {
	const isUser = message.sender === "user";
	const navigate = useNavigate();
	// @ts-ignore
	const skipToConversationDetail = () => {
		navigate("/conversation", {
			state: {
				question: message.text,
			},
		});
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
					"relative max-w-[70%] rounded-2xl px-4 py-2 shadow-sm backdrop-blur-md  bg-white/10 border border-white/10",
					isUser ? "text-white rounded-br-md" : "text-white rounded-bl-md",
				)}
			>
				{isUser ? (
					<p className="text-sm leading-relaxed break-words">{message.text}</p>
				) : (
					<div className={"flex flex-col"}>
						<p className="text-sm leading-relaxed break-words">
							{message.text}
						</p>
					</div>
				)}
			</div>
		</div>
	);
};
