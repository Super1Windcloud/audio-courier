import type React from "react";
import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import TitleBar from "@/components/TitleBar.tsx";
import useAppStateStore from "@/stores";
import { MessageInput } from "./MessageInput";
import { MessageList } from "./MessageList";

export interface Message {
	id: number;
	text: string;
	sender: "user" | "robot";
}

export const ChatContainer: React.FC = () => {
	const didRun = useRef(false);
	// 用 ref 存消息，避免 React 状态更新导致未更新完成的旧的状态丢失
	const messagesRef = useRef<Message[]>([
		{
			id: 0,
			text: "你好,请开始你的语音对话",
			sender: "robot",
		},
	]);

	const [messages, setMessages] = useState<Message[]>(messagesRef.current);
	const [isTyping, setIsTyping] = useState(false);
	const llmPromptStore = useAppStateStore((state) => state.llmPrompt);
	const currentSelectedModel = useAppStateStore(
		(state) => state.currentSelectedModel,
	);
	const uiOpacity = useAppStateStore((state) => state.uiOpacity);
	const uiTextTone = useAppStateStore((state) => state.uiTextTone);
	const licenseStatus = useAppStateStore((state) => state.licenseStatus);
	const isAuthorized = Boolean(
		licenseStatus?.isValid || licenseStatus?.isHostSigner,
	);
	const panelStyle = {
		borderColor: `rgb(255 255 255 / ${0.08 + uiOpacity * 0.12})`,
		background: `linear-gradient(180deg, rgb(114 71 102 / ${Math.max(0.24, uiOpacity * 0.72)}) 0%, rgb(44 79 113 / ${Math.max(0.18, uiOpacity * 0.62)}) 100%)`,
		transition:
			"background 180ms ease-out, border-color 180ms ease-out, box-shadow 180ms ease-out",
	};
	const footerStyle = {
		backgroundColor: `rgb(0 0 0 / ${Math.max(0.04, uiOpacity * 0.1)})`,
		transition: "background-color 180ms ease-out, border-color 180ms ease-out",
	};

	const updateSpecificBotMessage = useCallback(
		(id: number, content: string) => {
			const msgs = messagesRef.current;
			const idx = msgs.findIndex((msg) => msg.id === id);

			if (idx === -1) return;

			msgs[idx] = {
				...msgs[idx],
				text: content,
			};

			setMessages([...msgs]);
		},
		[],
	);

	const handleSendMessage = useCallback(
		async (text: string, introduceSelf?: boolean) => {
			if (!isAuthorized) {
				toast.warning("当前许可证无效，请先完成离线激活");
				return;
			}

			const userMsg: Message = {
				id: messagesRef.current.length,
				text,
				sender: "user",
			};
			messagesRef.current.push(userMsg);

			// 添加机器人占位
			const botMsg: Message = {
				id: messagesRef.current.length,
				text: "",
				sender: "robot",
			};
			messagesRef.current.push(botMsg);

			setMessages([...messagesRef.current]);

			const thisBotId = botMsg.id; // ← 记录本轮机器人消息 ID

			setIsTyping(true);

			const { llmInterviewChatStreamOutput } = await import("@/lib/llm.ts");
			await llmInterviewChatStreamOutput(
				text,
				introduceSelf ? import.meta.env.VITE_INTERVIEW_PROMPT : llmPromptStore,
				currentSelectedModel,
				(content) => {
					setIsTyping(false);
					updateSpecificBotMessage(thisBotId, content); // ← 更新特定机器人消息
				},
			);
		},
		[
			currentSelectedModel,
			isAuthorized,
			llmPromptStore,
			updateSpecificBotMessage,
		],
	);

	const handleClearConversation = () => {
		messagesRef.current = [
			{
				id: 0,
				text: "你好,请开始你的语音对话",
				sender: "robot",
			},
		];
		setMessages([...messagesRef.current]);
		setIsTyping(false);
	};

	useEffect(() => {
		if (didRun.current) return;
		didRun.current = true;
		if (import.meta.env.VITE_INIT_MESSAGE) {
			handleSendMessage(import.meta.env.VITE_INIT_MESSAGE, true).then();
		}
	}, [handleSendMessage]);

	return (
		<div className="flex h-screen w-screen">
			<div
				className={`flex h-full w-full flex-col overflow-hidden border backdrop-blur-xl ${
					uiTextTone === "dark" ? "ui-text-dark" : "ui-text-light"
				}`}
				style={panelStyle}
			>
				<div className="flex-shrink-0 border-b border-white/10">
					<TitleBar />
				</div>
				<div
					className="flex-1 w-full overflow-auto self-center"
					style={{
						overflow: "auto",
						scrollBehavior: "smooth",
						scrollbarWidth: "none",
					}}
				>
					{!isAuthorized ? (
						<div className="mx-4 mt-4 rounded-2xl border border-amber-300/20 bg-amber-400/10 p-4 text-sm text-amber-100">
							许可证状态: {licenseStatus?.reason ?? "未加载"}
							。点击顶部“许可证”生成设备请求码并导入授权。
						</div>
					) : null}
					<MessageList messages={messages} isTyping={isTyping} />
				</div>

				<div
					className="w-full flex-shrink-0 self-center border-t border-white/10"
					style={footerStyle}
				>
					<MessageInput
						onSendMessage={handleSendMessage}
						onClearConversation={handleClearConversation}
						setIsTyping={setIsTyping}
						setMessages={setMessages}
					/>
				</div>
			</div>
		</div>
	);
};
