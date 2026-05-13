import type React from "react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import { copyText } from "@/lib/clipboard";
import { cn } from "@/lib/utils";
import type { Message } from "./ChatContainer";
import { MarkdownMessage } from "./MarkdownMessage";

interface MessageItemProps {
	message: Message;
	onDeleteMessage: (id: number) => void;
}

interface ContextMenuPosition {
	x: number;
	y: number;
}

export const MessageItem: React.FC<MessageItemProps> = ({
	message,
	onDeleteMessage,
}) => {
	const isUser = message.sender === "user";
	const [contextMenuPosition, setContextMenuPosition] =
		useState<ContextMenuPosition | null>(null);

	const closeContextMenu = useCallback(() => {
		setContextMenuPosition(null);
	}, []);

	const handleContextMenu = useCallback(
		(event: React.MouseEvent<HTMLDivElement>) => {
			event.preventDefault();
			event.stopPropagation();
			setContextMenuPosition({
				x: event.clientX,
				y: event.clientY,
			});
		},
		[],
	);

	const handleCopy = useCallback(async () => {
		closeContextMenu();
		const didCopy = await copyText(message.text);

		if (didCopy) {
			toast.success("已复制消息");
			return;
		}

		toast.warning("当前环境不允许自动复制，请手动复制消息内容");
	}, [closeContextMenu, message.text]);

	const handleDelete = useCallback(() => {
		closeContextMenu();
		onDeleteMessage(message.id);
	}, [closeContextMenu, message.id, onDeleteMessage]);

	useEffect(() => {
		if (!contextMenuPosition) return;

		window.addEventListener("click", closeContextMenu);
		window.addEventListener("contextmenu", closeContextMenu);
		window.addEventListener("blur", closeContextMenu);
		window.addEventListener("scroll", closeContextMenu, true);

		return () => {
			window.removeEventListener("click", closeContextMenu);
			window.removeEventListener("contextmenu", closeContextMenu);
			window.removeEventListener("blur", closeContextMenu);
			window.removeEventListener("scroll", closeContextMenu, true);
		};
	}, [closeContextMenu, contextMenuPosition]);

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
				onContextMenu={handleContextMenu}
			>
				{isUser ? (
					<p className="text-sm leading-relaxed break-words">{message.text}</p>
				) : (
					<div className={"flex flex-col"}>
						<MarkdownMessage content={message.text} />
					</div>
				)}
			</div>
			{contextMenuPosition ? (
				<div
					className="fixed z-50 min-w-28 overflow-hidden rounded-xl border border-white/10 bg-slate-900/95 p-1 text-sm text-white shadow-2xl backdrop-blur-md"
					style={{
						left: contextMenuPosition.x,
						top: contextMenuPosition.y,
					}}
					onClick={(event) => event.stopPropagation()}
					onContextMenu={(event) => {
						event.preventDefault();
						event.stopPropagation();
					}}
				>
					<button
						type="button"
						className="block w-full rounded-lg px-3 py-2 text-left transition-colors hover:bg-white/10 focus:bg-white/10 focus:outline-none"
						onClick={() => {
							void handleCopy();
						}}
					>
						复制
					</button>
					<button
						type="button"
						className="block w-full rounded-lg px-3 py-2 text-left text-red-200 transition-colors hover:bg-red-500/20 focus:bg-red-500/20 focus:outline-none"
						onClick={handleDelete}
					>
						删除
					</button>
				</div>
			) : null}
		</div>
	);
};
