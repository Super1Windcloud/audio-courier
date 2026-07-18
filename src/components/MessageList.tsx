import type React from "react";
import { useEffect, useRef, useState } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import useAppStateStore from "@/stores";
import type { Message } from "./ChatContainer";
import { MessageItem } from "./MessageItem";

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
	const [openContextMenuMessageId, setOpenContextMenuMessageId] = useState<
		number | null
	>(null);
	const isScrolling = useAppStateStore((state) => state.isStartScrollToBottom);
	const hasPendingRobotMessage = messages.some(
		(message) => message.sender === "robot" && message.text.trim() === "",
	);

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
		<ScrollArea ref={scrollAreaRef} className="h-full px-4">
			<div className="space-y-4 py-4">
				{messages.map((message) => (
					<MessageItem
						key={message.id}
						message={message}
						contextMenuOpen={openContextMenuMessageId === message.id}
						onContextMenuOpen={() => setOpenContextMenuMessageId(message.id)}
						onContextMenuClose={() =>
							setOpenContextMenuMessageId((currentId) =>
								currentId === message.id ? null : currentId,
							)
						}
						onDeleteMessage={onDeleteMessage}
					/>
				))}
				{isTyping && !hasPendingRobotMessage && (
					<MessageItem
						message={{ id: -1, text: "", sender: "robot" }}
						contextMenuOpen={openContextMenuMessageId === -1}
						onContextMenuOpen={() => setOpenContextMenuMessageId(-1)}
						onContextMenuClose={() =>
							setOpenContextMenuMessageId((currentId) =>
								currentId === -1 ? null : currentId,
							)
						}
						onDeleteMessage={onDeleteMessage}
					/>
				)}
			</div>
		</ScrollArea>
	);
};
