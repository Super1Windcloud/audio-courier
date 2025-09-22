import React, { useEffect, useRef } from "react";
import { MessageItem } from "./MessageItem";
import { TypingIndicator } from "./TypingIndicator";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { Message } from "./ChatContainer";
import useAppStateStore from "@/stores";

interface MessageListProps {
	messages: Message[];
	isTyping: boolean;
}

export const MessageList: React.FC<MessageListProps> = ({
	messages,
	isTyping,
}) => {
	const messagesEndRef = useRef<HTMLDivElement>(null);
	const isScrolling = useAppStateStore((state) => state.isStartScrollToBottom);
	const scrollToBottom = () => {
		messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
	};

	useEffect(() => {
		if (isScrolling) {
			scrollToBottom();
		}
	}, [messages, isTyping]);

	return (
		<ScrollArea className="h-full px-4">
			<div className="space-y-4 py-4">
				{messages.map((message) => (
					<MessageItem key={message.id} message={message} />
				))}
				{isTyping && <TypingIndicator />}
				<div ref={messagesEndRef} />
			</div>
		</ScrollArea>
	);
};
