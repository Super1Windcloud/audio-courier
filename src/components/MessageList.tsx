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
		const shouldScroll = isScrolling && messages.length > 0;

		if (!shouldScroll) return;

		const scrollViewport = scrollAreaRef.current?.querySelector(
			"[data-radix-scroll-area-viewport]",
		);

		if (!(scrollViewport instanceof HTMLDivElement)) return;

		scrollViewport.scrollTo({
			top: scrollViewport.scrollHeight,
			behavior: "smooth",
		});
	}, [isScrolling, messages]);

	return (
		<div className="relative h-full">
			<ScrollArea ref={scrollAreaRef} className="h-full px-4">
				<div className="space-y-4 py-4">
					{messages.map((message) => (
						<MessageItem
							key={message.id}
							message={message}
							onDeleteMessage={onDeleteMessage}
						/>
					))}
				</div>
			</ScrollArea>
			{isTyping && (
				<div className="pointer-events-none absolute bottom-4 left-4 z-10">
					<TypingIndicator />
				</div>
			)}
		</div>
	);
};
