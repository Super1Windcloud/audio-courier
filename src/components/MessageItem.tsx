import type React from "react";

interface MessageItemProps {
	username: string;
	message: string;
	time: string;
}

const MessageItem: React.FC<MessageItemProps> = ({
	username,
	message,
	time,
}) => {
	return (
		<div className="flex  items-start gap-3 p-2 hover:bg-gray-50 rounded-md">
			{/* 头像首字母 */}
			<div className="w-8 h-8 flex items-center justify-center rounded-full bg-blue-500 text-white font-bold">
				{username.charAt(0)}
			</div>

			{/* 消息内容 */}
			<div className="flex-1">
				{/* 用户名和时间 */}
				<div className="flex items-center justify-between">
					<span className="font-semibold text-gray-800">{username}</span>
					<span className="text-xs text-gray-400">{time}</span>
				</div>

				{/* 消息文本 */}
				<div className="mt-1 text-gray-700">{message}</div>
			</div>
		</div>
	);
};

export default MessageItem;
