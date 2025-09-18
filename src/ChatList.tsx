import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useEffect, useRef, useState } from "react";

interface Message {
	id: number;
	role: "user" | "bot";
	content: string;
	time: string;
	avatar?: string;
}

const initialMessages: Message[] = [
	{
		id: 1,
		role: "bot",
		content: "你好！欢迎使用 Telegram 风格聊天。",
		time: "10:00",
		avatar: "/bot.png",
	},
	{
		id: 2,
		role: "bot",
		content: "你好！欢迎使用 Telegram 风格聊天。",
		time: "10:00",
		avatar: "/bot.png",
	},
	{
		id: 3,
		role: "bot",
		content: "你好！欢迎使用 Telegram 风格聊天。",
		time: "10:00",
		avatar: "/bot.png",
	},
	{
		id: 4,
		role: "bot",
		content: "你好！欢迎使用 Telegram 风格聊天。",
		time: "10:00",
		avatar: "/bot.png",
	},
];

const ChatTelegram = () => {
	const [messages, setMessages] = useState<Message[]>(initialMessages);
	const [inputValue, setInputValue] = useState("");
	const bottomRef = useRef<HTMLDivElement>(null);

	useEffect(() => {
		bottomRef.current?.scrollIntoView({ behavior: "smooth" });
	}, [messages]);

	const handleSend = () => {
		if (!inputValue.trim()) return;

		const newMessage: Message = {
			id: messages.length + 1,
			role: "user",
			content: inputValue,
			time: new Date().toLocaleTimeString([], {
				hour: "2-digit",
				minute: "2-digit",
			}),
			avatar: "/user.png",
		};

		setMessages([...messages, newMessage]);
		setInputValue("");

		// 模拟机器人回复
		setTimeout(() => {
			const botMessage: Message = {
				id: messages.length + 2,
				role: "bot",
				content: "收到：" + newMessage.content,
				time: new Date().toLocaleTimeString([], {
					hour: "2-digit",
					minute: "2-digit",
				}),
				avatar: "/bot.png",
			};
			setMessages((prev) => [...prev, botMessage]);
		}, 800);
	};

	return (
		<div className="flex flex-col h-full bg-transparent">
			{/* 聊天列表 */}
			<div className="flex-1 overflow-y-auto p-4 flex flex-col gap-3">
				{messages.map((msg) => (
					<div
						key={msg.id}
						className={`flex items-end gap-3 ${msg.role === "user" ? "justify-end" : "justify-start"}`}
					>
						{msg.role === "bot" && (
							<Avatar className="w-8 h-8">
								<AvatarImage src={msg.avatar} />
								<AvatarFallback>🤖</AvatarFallback>
							</Avatar>
						)}

						<Card
							className={`max-w-xs shadow-md rounded-2xl p-2 ${
								msg.role === "user"
									? "bg-blue-500 text-white"
									: "bg-gray-700 text-white"
							} bg-opacity-80`}
						>
							<CardContent className="p-2">
								<p className="text-sm break-words">{msg.content}</p>
								<span className="text-xs text-gray-300 mt-1 block text-right">
									{msg.time}
								</span>
							</CardContent>
						</Card>

						{msg.role === "user" && (
							<Avatar className="w-8 h-8">
								<AvatarImage src={msg.avatar} />
								<AvatarFallback>👤</AvatarFallback>
							</Avatar>
						)}
					</div>
				))}
				<div ref={bottomRef} />
			</div>

			{/* 输入框 */}
			<div className="flex p-4 gap-2 bg-transparent border-t border-gray-600">
				<Input
					className="flex-1 bg-gray-800 text-white placeholder-gray-400 rounded-full"
					placeholder="输入消息..."
					value={inputValue}
					onChange={(e) => setInputValue(e.target.value)}
					onKeyDown={(e) => e.key === "Enter" && handleSend()}
				/>
				<Button onClick={handleSend} className="rounded-full px-4">
					发送
				</Button>
			</div>
		</div>
	);
};

export default ChatTelegram;
