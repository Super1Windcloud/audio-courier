import type React from "react";
import { useEffect, useRef } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import useAppStateStore from "@/stores";
import type { Message } from "./ChatContainer";
import { MessageItem } from "./MessageItem";
import { TypingIndicator } from "./TypingIndicator";

interface MessageListProps {
	messages: Message[];
	isTyping: boolean;
	onDeleteMessage: (id: number) => void;
}

export const MessageList: React.FC<MessageListProps> = ({
	messages,
	isTyping,
	onDeleteMessage,
}) => {
	const scrollAreaRef = useRef<HTMLDivElement>(null);
	const isScrolling = useAppStateStore((state) => state.isStartScrollToBottom);

	useEffect(() => {
		const shouldScroll = isScrolling && (messages.length > 0 || isTyping);

		if (!shouldScroll) return;

		const scrollViewport = scrollAreaRef.current?.querySelector(
			"[data-radix-scroll-area-viewport]",
		);

		if (!(scrollViewport instanceof HTMLDivElement)) return;

		scrollViewport.scrollTo({
			top: scrollViewport.scrollHeight,
			behavior: "smooth",
		});
	}, [isScrolling, isTyping, messages]);

	return (
		<ScrollArea
			ref={scrollAreaRef}
			className="h-full px-4"
			style={{
				scrollbarWidth: "none",
			}}
		>
			<div
				style={{
					scrollbarWidth: "none",
				}}
				className="space-y-4 py-4"
			>
				{messages.map((message) => (
					<MessageItem
						key={message.id}
						message={message}
						onDeleteMessage={onDeleteMessage}
					/>
				))}
				{isTyping && <TypingIndicator />}
			</div>
		</ScrollArea>
	);
};
